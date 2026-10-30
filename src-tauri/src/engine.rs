use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

mod commands;
pub use commands::CommandSpec;
mod ports;
mod removal;
pub use ports::ensure_port_free;
pub use removal::{RemovalOptions, RemovalPreview};

pub type Result<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "legacy_release")]
    pub version: String,
    pub id: String,
    pub name: String,
    pub recipe: String,
    pub port: u16,
    pub command: String,
    #[serde(default)]
    pub app_id: Option<String>,
    #[serde(default)]
    pub run: Option<CommandSpec>,
    #[serde(default)]
    pub install_steps: Vec<CommandSpec>,
    pub cwd: String,
    pub protocol: String,
    pub health_path: String,
    pub tier: String,
    pub model_source: String,
    #[serde(default)]
    pub installed: bool,
    #[serde(default)]
    pub models_ready: bool,
}
fn legacy_release() -> String {
    "4.1.2".into()
}

fn release(version: &str) -> Result<Value> {
    let catalog: Value =
        serde_json::from_str(include_str!("../../registry/mineru.json")).map_err(err)?;
    catalog["releases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["version"] == version)
        .cloned()
        .ok_or("该 Release 尚无经过适配的安装配方".into())
}

impl Config {
    pub fn mineru() -> Self {
        Self {
            version: legacy_release(),
            id: "mineru".into(),
            name: "MinerU".into(),
            recipe: "mineru".into(),
            port: 8000,
            command: String::new(),
            app_id: None,
            run: None,
            install_steps: Vec::new(),
            cwd: String::new(),
            protocol: "http-api".into(),
            health_path: "/v1/health".into(),
            tier: "standard".into(),
            model_source: "huggingface".into(),
            installed: false,
            models_ready: false,
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.recipe == "mineru" {
            release(&self.version)?;
        }
        if self.name.trim().is_empty() || self.name.len() > 120 {
            return Err("请填写服务名称（最多 120 字符）".into());
        }
        if self.port < 1024 {
            return Err("端口需要在 1024–65535 之间".into());
        }
        if !["http-api", "mcp-http"].contains(&self.protocol.as_str()) {
            return Err("不支持的协议".into());
        }
        if !self.health_path.is_empty()
            && (!self.health_path.starts_with('/')
                || self.health_path.starts_with("//")
                || self.health_path.contains(['\n', '\r', '\\', '#']))
        {
            return Err("健康检查路径需要是 / 开头的本地路径".into());
        }
        if !["standard", "basic", "flash"].contains(&self.tier.as_str())
            || !["huggingface", "modelscope"].contains(&self.model_source.as_str())
        {
            return Err("不支持的部署配置".into());
        }
        if let Some(run) = &self.run {
            run.validate()?;
            if !self.command.is_empty() {
                return Err("不能同时设置结构化 run 与 shell command".into());
            }
        }
        if self.install_steps.len() > 32 {
            return Err("安装步骤不能超过 32 项".into());
        }
        for step in &self.install_steps {
            step.validate()?;
        }
        if self.recipe != "mineru"
            && ((self.run.is_none() && self.command.trim().is_empty())
                || !Path::new(&self.cwd).is_dir())
        {
            return Err("请填写启动命令并选择存在的工作目录".into());
        }
        Ok(())
    }
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub config: Config,
    pub status: String,
    pub stage: String,
    pub error: Option<String>,
    pub pid: Option<u32>,
    pub started_at: Option<u64>,
    pub endpoint: String,
    pub command_preview: String,
}
struct Entry {
    view: View,
    cancel: Arc<AtomicBool>,
    busy: bool,
    removing: bool,
}
#[derive(Clone)]
pub struct Engine {
    root: PathBuf,
    uv: PathBuf,
    inner: Arc<Mutex<BTreeMap<String, Entry>>>,
    closing: Arc<AtomicBool>,
}

impl Engine {
    pub fn new(root: PathBuf, uv: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root).map_err(err)?;
        let configs: Vec<Config> = if root.join("services.json").exists() {
            serde_json::from_slice(&fs::read(root.join("services.json")).map_err(err)?)
                .map_err(|e| format!("服务配置损坏，未覆盖原文件：{e}"))?
        } else {
            vec![Config::mineru()]
        };
        let engine = Self {
            root,
            uv,
            inner: Arc::new(Mutex::new(BTreeMap::new())),
            closing: Arc::new(AtomicBool::new(false)),
        };
        for mut c in configs {
            c.validate()?;
            if c.recipe == "mineru" && c.name == "MinerU" {
                c.name = format!("MinerU {}", c.version);
            }
            if !c.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') || c.id.is_empty() {
                return Err("无效的服务 ID".into());
            }
            let mut view = engine.make_view(c);
            if engine
                .instance_dir(&view.config.id)
                .join("active.json")
                .exists()
            {
                view.status = "interrupted".into();
                view.error=Some("上次 Launcher 未正常退出。请查看日志并确认旧进程已结束，再点击清除中断记录。不会根据旧 PID 自动终止进程。".into());
            }
            engine.inner.lock().unwrap().insert(
                view.config.id.clone(),
                Entry {
                    view,
                    cancel: Arc::new(AtomicBool::new(false)),
                    busy: false,
                    removing: false,
                },
            );
        }
        engine.save()?;
        Ok(engine)
    }
    pub fn root(&self) -> String {
        self.root.display().to_string()
    }
    pub fn catalog(&self) -> Value {
        serde_json::from_str(include_str!("../../registry/mineru.json")).unwrap()
    }
    pub fn views(&self) -> Vec<View> {
        self.inner
            .lock()
            .unwrap()
            .values()
            .map(|e| e.view.clone())
            .collect()
    }
    pub fn view(&self, id: &str) -> Result<View> {
        self.inner
            .lock()
            .unwrap()
            .get(id)
            .map(|e| e.view.clone())
            .ok_or("找不到服务".into())
    }
    fn instance_dir(&self, id: &str) -> PathBuf {
        self.root.join("instances").join(id)
    }
    pub fn log_path(&self, id: &str) -> Result<PathBuf> {
        self.view(id)?;
        Ok(self.instance_dir(id).join("service.log"))
    }
    fn env_dir(&self, c: &Config) -> PathBuf {
        self.root
            .join("apps")
            .join(&c.recipe)
            .join(&c.version)
            .join("env")
    }
    fn bin(&self, c: &Config, name: &str) -> PathBuf {
        if cfg!(windows) {
            self.env_dir(c).join("Scripts").join(format!("{name}.exe"))
        } else {
            self.env_dir(c).join("bin").join(name)
        }
    }
    fn make_view(&self, c: Config) -> View {
        let preview = if c.recipe == "mineru" {
            format!(
                "mineru-kit api-server --host 127.0.0.1 --port {} --tier {}",
                c.port, c.tier
            )
        } else if let Some(run) = &c.run {
            run.preview()
        } else {
            c.command.replace("${port}", &c.port.to_string())
        };
        let endpoint = format!(
            "http://127.0.0.1:{}{}",
            c.port,
            if c.protocol == "mcp-http" {
                c.health_path.as_str()
            } else {
                ""
            }
        );
        View {
            config: c,
            status: "stopped".into(),
            stage: String::new(),
            error: None,
            pid: None,
            started_at: None,
            endpoint,
            command_preview: preview,
        }
    }
    fn save(&self) -> Result<()> {
        let guard = self.inner.lock().unwrap();
        self.save_entries(&guard)
    }
    fn save_entries(&self, guard: &BTreeMap<String, Entry>) -> Result<()> {
        let configs: Vec<_> = guard.values().map(|e| &e.view.config).collect();
        let data = serde_json::to_vec_pretty(&configs).map_err(err)?;
        let path = self.root.join("services.json");
        let tmp = path.with_extension("json.tmp");
        let mut f = File::create(&tmp).map_err(err)?;
        f.write_all(&data).map_err(err)?;
        f.sync_all().map_err(err)?;
        #[cfg(windows)]
        if path.exists() {
            fs::copy(&path, path.with_extension("json.bak")).map_err(err)?;
            fs::remove_file(&path).map_err(err)?;
        }
        fs::rename(tmp, path).map_err(err)?;
        Ok(())
    }
    pub fn upsert(&self, mut c: Config) -> Result<String> {
        if c.id.is_empty() {
            c.recipe = "custom".into();
        }
        {
            let mut inner = self.inner.lock().unwrap();
            if c.id.is_empty() {
                c.id = uuid::Uuid::new_v4().to_string();
                c.recipe = "custom".into();
                c.installed = c.install_steps.is_empty();
                c.models_ready = true;
            } else {
                let old = inner.get(&c.id).ok_or("找不到服务")?;
                if old.busy || old.removing || old.view.status == "interrupted" {
                    return Err("请先停止服务，再修改配置".into());
                }
                c.recipe = old.view.config.recipe.clone();
                if c.version != old.view.config.version {
                    return Err("请在应用市场选择另一个 Release；已有实例的版本不可覆盖".into());
                }
                c.installed = old.view.config.installed;
                if c.recipe != "mineru"
                    && (c.cwd != old.view.config.cwd
                        || c.install_steps != old.view.config.install_steps)
                {
                    c.installed = c.install_steps.is_empty();
                }
                c.models_ready = old.view.config.models_ready && c.tier == old.view.config.tier;
            }
            c.validate()?;
            let view = self.make_view(c.clone());
            inner.insert(
                c.id.clone(),
                Entry {
                    view,
                    cancel: Arc::new(AtomicBool::new(false)),
                    busy: false,
                    removing: false,
                },
            );
        }
        self.save()?;
        Ok(c.id)
    }
    /// Select an adapted release without replacing another installed version.
    pub fn add_release(&self, recipe: &str, version: &str) -> Result<String> {
        if recipe != "mineru" {
            return Err("未知的应用配方".into());
        }
        release(version)?;
        let mut inner = self.inner.lock().unwrap();
        if let Some(e) = inner
            .values()
            .find(|e| e.view.config.recipe == recipe && e.view.config.version == version)
        {
            if e.removing {
                return Err("该服务正在删除".into());
            }
            return Ok(e.view.config.id.clone());
        }
        let mut c = Config::mineru();
        c.version = version.into();
        c.id = format!("{recipe}-{}", version.replace('.', "-"));
        c.name = format!("MinerU {version}");
        // Reserve a distinct configured port, also respecting external listeners.
        c.port = (8000..=65535)
            .find(|port| {
                !inner.values().any(|e| e.view.config.port == *port)
                    && ensure_port_free(*port).is_ok()
            })
            .ok_or("没有可用端口")?;
        let id = c.id.clone();
        inner.insert(
            id.clone(),
            Entry {
                view: self.make_view(c),
                cancel: Arc::new(AtomicBool::new(false)),
                busy: false,
                removing: false,
            },
        );
        drop(inner);
        self.save()?;
        Ok(id)
    }
    pub fn remove(&self, id: &str) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        let e = inner.get(id).ok_or("找不到服务")?;
        if e.busy || e.removing || e.view.status == "interrupted" {
            return Err("请先停止或处理上次中断".into());
        }
        let removed = inner.remove(id).unwrap();
        // Persist while holding the registry lock. If saving fails, leave the
        // service available in memory as well as in the previous on-disk list.
        if let Err(error) = self.save_entries(&inner) {
            inner.insert(id.to_owned(), removed);
            return Err(error);
        }
        Ok(())
    }
    pub fn acknowledge(&self, id: &str) -> Result<()> {
        let v = self.view(id)?;
        if v.status != "interrupted" {
            return Err("服务没有中断记录".into());
        }
        ensure_port_free(v.config.port)?;
        fs::remove_file(self.instance_dir(id).join("active.json")).map_err(err)?;
        self.update(id, |e| {
            e.view.status = "stopped".into();
            e.view.error = None;
        });
        Ok(())
    }
    fn update(&self, id: &str, f: impl FnOnce(&mut Entry)) {
        if let Some(e) = self.inner.lock().unwrap().get_mut(id) {
            f(e)
        }
    }
    fn reserve(&self, id: &str, status: &str) -> Result<(Config, Arc<AtomicBool>)> {
        if self.closing.load(Ordering::SeqCst) {
            return Err("Launcher 正在退出".into());
        }
        let mut guard = self.inner.lock().unwrap();
        let e = guard.get_mut(id).ok_or("找不到服务")?;
        if e.busy || e.removing || e.view.status == "interrupted" {
            return Err("服务正在执行操作或需要处理上次中断".into());
        }
        e.busy = true;
        e.cancel = Arc::new(AtomicBool::new(false));
        e.view.status = status.into();
        e.view.error = None;
        e.view.stage.clear();
        Ok((e.view.config.clone(), e.cancel.clone()))
    }
    fn finish(&self, id: &str, result: Result<()>, cancelled: bool) {
        // Keep the operation reserved until its marker is removed.
        let _ = fs::remove_file(self.instance_dir(id).join("active.json"));
        self.update(id, |e| {
            e.busy = false;
            e.view.pid = None;
            e.view.started_at = None;
            e.view.stage.clear();
            match result {
                Ok(()) => {
                    e.view.status = "stopped".into();
                    e.view.error = None;
                }
                Err(ref s) if cancelled => {
                    e.view.status = "stopped".into();
                    e.view.error = Some(format!("操作已取消：{s}"));
                }
                Err(s) => {
                    e.view.status = "error".into();
                    e.view.error = Some(s);
                }
            }
        });
        let _ = self.save();
    }
    fn log(&self, id: &str, line: &str) {
        let dir = self.instance_dir(id);
        let _ = fs::create_dir_all(&dir);
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("service.log"))
        {
            let _ = writeln!(f, "[{}] {}", now(), line);
        }
    }
    fn log_file(&self, id: &str) -> Result<File> {
        let dir = self.instance_dir(id);
        fs::create_dir_all(&dir).map_err(err)?;
        let p = dir.join("service.log");
        if fs::metadata(&p)
            .map(|m| m.len() > 8 * 1024 * 1024)
            .unwrap_or(false)
        {
            let _ = fs::rename(&p, dir.join("previous.log"));
        }
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
            .map_err(err)
    }
    pub fn logs(&self, id: &str) -> Result<String> {
        let p = self.log_path(id)?;
        if !p.exists() {
            return Ok("尚无日志。安装或启动后，输出会显示在这里。".into());
        }
        let mut f = File::open(p).map_err(err)?;
        let len = f.metadata().map_err(err)?.len();
        f.seek(SeekFrom::Start(len.saturating_sub(64 * 1024)))
            .map_err(err)?;
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes).map_err(err)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
    fn process_env(&self, c: &Config, cmd: &mut Command) -> Result<()> {
        // Transformers 5 async safetensors loading can deadlock during cold
        // model initialization on macOS. Set this before Python imports it.
        // Scope the workaround to managed MinerU; custom commands keep their env.
        if cfg!(target_os = "macos") && c.recipe == "mineru" {
            cmd.env("HF_DEACTIVATE_ASYNC_LOAD", "1");
        }
        let data = self.instance_dir(&c.id).join("data");
        fs::create_dir_all(&data).map_err(err)?;
        fs::create_dir_all(self.root.join("cache").join(&c.id)).map_err(err)?;
        // Tool-specific managed environment belongs to the MinerU adapter only.
        if c.recipe == "mineru" {
            cmd.env("PYTHONUNBUFFERED", "1")
                .env("PYTHONNOUSERSITE", "1")
                .env("UV_CACHE_DIR", self.root.join("cache/uv"))
                .env("UV_PYTHON_INSTALL_DIR", self.root.join("runtimes/python"))
                .env("UV_PYTHON_PREFERENCE", "only-managed")
                .env("HF_HOME", self.root.join("cache/huggingface"))
                .env("MODELSCOPE_CACHE", self.root.join("cache/modelscope"))
                .env("MINERU_HOME", data.join("mineru"))
                .env("MINERU_CONFIG", data.join("mineru/config.yaml"));
        }
        cmd.env("LAUNCHER_DATA_DIR", &data)
            .env("LAUNCHER_CACHE_DIR", self.root.join("cache").join(&c.id))
            .env("LAUNCHER_INSTANCE_ID", &c.id)
            .env("LAUNCHER_PORT", c.port.to_string())
            .env("LAUNCHER_HOST", "127.0.0.1");
        Ok(())
    }
    fn spawn(&self, c: &Config, cmd: &mut Command) -> Result<Child> {
        self.process_env(c, cmd)?;
        let f = self.log_file(&c.id)?;
        cmd.stdin(Stdio::null())
            .stdout(f.try_clone().map_err(err)?)
            .stderr(f);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x00000200 | 0x08000000);
        }
        let mut child = cmd.spawn().map_err(|e| format!("无法执行命令：{e}"))?;
        self.update(&c.id, |e| e.view.pid = Some(child.id()));
        if let Err(e) = fs::write(
            self.instance_dir(&c.id).join("active.json"),
            json!({"pid":child.id(),"startedAt":now(),"port":c.port}).to_string(),
        ) {
            terminate(&mut child);
            return Err(err(e));
        }
        Ok(child)
    }
    fn step(&self, c: &Config, cancel: &AtomicBool, label: &str, cmd: &mut Command) -> Result<()> {
        if cancel.load(Ordering::SeqCst) {
            return Err("任务已取消".into());
        }
        self.update(&c.id, |e| e.view.stage = label.into());
        self.log(&c.id, label);
        let mut child = self.spawn(c, cmd)?;
        let start = Instant::now();
        loop {
            if cancel.load(Ordering::SeqCst) || start.elapsed() > Duration::from_secs(7200) {
                terminate(&mut child);
                return Err("任务取消或达到两小时超时限制".into());
            }
            if let Some(exit) = match child.try_wait() {
                Ok(value) => value,
                Err(e) => {
                    terminate(&mut child);
                    return Err(err(e));
                }
            } {
                cleanup_group(child.id());
                return if exit.success() {
                    Ok(())
                } else {
                    Err(format!("{label}失败（{exit}），请查看日志"))
                };
            }
            thread::sleep(Duration::from_millis(200));
        }
    }
    pub fn install(&self, id: &str) -> Result<()> {
        let config = self.view(id)?.config;
        if config.recipe != "mineru" && config.install_steps.is_empty() {
            return Err("此服务没有声明安装步骤".into());
        }
        let (c, cancel) = self.reserve(id, "installing")?;
        let engine = self.clone();
        thread::spawn(move || {
            let result = engine.install_inner(&c, &cancel);
            if let Err(ref e) = result {
                engine.log(&c.id, e)
            };
            engine.finish(&c.id, result, cancel.load(Ordering::SeqCst));
        });
        Ok(())
    }
    fn install_inner(&self, c: &Config, cancel: &AtomicBool) -> Result<()> {
        if c.recipe != "mineru" {
            self.update(&c.id, |e| e.view.config.installed = false);
            self.save()?;
            for (index, spec) in c.install_steps.iter().enumerate() {
                self.step(
                    c,
                    cancel,
                    &format!("安装步骤 {}/{}", index + 1, c.install_steps.len()),
                    &mut self.structured_command(c, spec)?,
                )?;
            }
            if cancel.load(Ordering::SeqCst) {
                return Err("安装已取消".into());
            }
            self.update(&c.id, |e| e.view.config.installed = true);
            if let Err(error) = self.save() {
                self.update(&c.id, |e| e.view.config.installed = false);
                return Err(error);
            }
            self.log(&c.id, "安装完成。点击启动服务。");
            return Ok(());
        }
        if !self.uv.is_file() {
            return Err("缺少随应用附带的 uv 运行时，请重新构建安装包".into());
        }
        let spec = release(&c.version)?;
        let python = spec["python"].as_str().ok_or("Release 缺少 Python 版本")?;
        let cfg_dir = self.instance_dir(&c.id).join("data/mineru");
        fs::create_dir_all(&cfg_dir).map_err(err)?;
        let models = self.model_dir(c);
        fs::create_dir_all(&models).map_err(err)?;
        let config =
            serde_yaml::to_string(&json!({"model":{"source":c.model_source,"base_dir":models}}))
                .map_err(err)?;
        fs::write(cfg_dir.join("config.yaml"), config).map_err(err)?;
        if !self.bin(c, "python").exists() {
            self.step(
                c,
                cancel,
                &format!("准备 Python {python} 独立环境"),
                Command::new(&self.uv)
                    .args(["venv", "--python", python])
                    .arg(self.env_dir(c)),
            )?;
        }
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            let lock = self.env_dir(c).parent().unwrap().join("requirements.lock");
            fs::write(
                &lock,
                match c.version.as_str() {
                    "4.0.12" => include_str!("../../registry/mineru-4.0.12-macos-arm64.lock"),
                    "4.1.2" => include_str!("../../registry/mineru-macos-arm64.lock"),
                    _ => return Err("缺少该 Release 的依赖锁".into()),
                },
            )
            .map_err(err)?;
            self.step(
                c,
                cancel,
                &format!("安装 MinerU {} 与固定依赖", c.version),
                Command::new(&self.uv)
                    .args(["pip", "sync", "--python"])
                    .arg(self.bin(c, "python"))
                    .arg(&lock),
            )?;
        }
        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        {
            return Err(
                "此 alpha 的 MinerU 配方仅在 Apple Silicon 上准备；其他平台尚待验证".into(),
            );
        }
        self.update(&c.id, |e| e.view.config.installed = true);
        self.save()?;
        if c.tier != "flash" {
            self.step(
                c,
                cancel,
                "下载解析模型（首次需要较长时间）",
                Command::new(self.bin(c, "mineru-kit")).args([
                    "models",
                    "download",
                    "--tier",
                    &c.tier,
                    "--source",
                    &c.model_source,
                ]),
            )?;
            self.step(
                c,
                cancel,
                "校验模型完整性",
                Command::new(self.bin(c, "mineru-kit"))
                    .args(["models", "verify", "--tier", &c.tier]),
            )?;
        }
        self.update(&c.id, |e| e.view.config.models_ready = true);
        self.save()?;
        self.log(&c.id, "部署完成。点击启动以运行本地服务。");
        Ok(())
    }
    pub fn start(&self, id: &str) -> Result<()> {
        let v = self.view(id)?;
        if !v.config.installed {
            return Err("请先安装服务".into());
        }
        if v.config.recipe == "mineru" && !v.config.models_ready {
            return Err("请先完成模型准备；可点击继续部署".into());
        }
        // Reserve first: a repeated start must not bind the port while the first
        // worker is performing its own preflight or starting the real server.
        let (c, cancel) = self.reserve(id, "starting")?;
        if let Err(error) = ensure_port_free(c.port) {
            self.finish(id, Err(error.clone()), false);
            return Err(error);
        }
        let engine = self.clone();
        thread::spawn(move || {
            let result = engine.run(&c, &cancel);
            if let Err(ref e) = result {
                engine.log(&c.id, e)
            };
            engine.finish(&c.id, result, cancel.load(Ordering::SeqCst));
        });
        Ok(())
    }
    fn run(&self, c: &Config, cancel: &AtomicBool) -> Result<()> {
        ensure_port_free(c.port)?;
        if c.recipe == "mineru" && c.tier != "flash" {
            if let Err(error) = self.step(
                c,
                cancel,
                "检查本地模型文件",
                Command::new(self.bin(c, "mineru-kit"))
                    .args(["models", "verify", "--tier", &c.tier]),
            ) {
                if !cancel.load(Ordering::SeqCst) {
                    self.update(&c.id, |e| e.view.config.models_ready = false);
                }
                return Err(error);
            }
            self.update(&c.id, |e| e.view.stage = "启动本地 API".into());
        }

        let mut cmd = if c.recipe == "mineru" {
            let mut cmd = Command::new(self.bin(c, "mineru-kit"));
            cmd.args([
                "api-server",
                "--host",
                "127.0.0.1",
                "--port",
                &c.port.to_string(),
                "--tier",
                &c.tier,
            ]);
            cmd.current_dir(self.instance_dir(&c.id));
            cmd
        } else if let Some(run) = &c.run {
            self.structured_command(c, run)?
        } else {
            let expanded = c.command.replace("${port}", &c.port.to_string());
            let mut cmd = if cfg!(windows) {
                let mut x = Command::new("cmd.exe");
                x.args(["/D", "/S", "/C", &expanded]);
                x
            } else {
                let mut x = Command::new("/bin/zsh");
                if !Path::new("/bin/zsh").exists() {
                    x = Command::new("/bin/sh");
                }
                x.args(["-lc", &expanded]);
                x
            };
            cmd.current_dir(&c.cwd);
            cmd
        };
        if cfg!(target_os = "macos") && c.recipe == "mineru" {
            self.log(&c.id, "模型权重串行加载：HF_DEACTIVATE_ASYNC_LOAD=1");
        }
        self.log(&c.id, &format!("启动 {}，端口 {}", c.name, c.port));
        let log_path = self.instance_dir(&c.id).join("service.log");
        let log_offset = fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0);
        let mut child = self.spawn(c, &mut cmd)?;
        let beginning = Instant::now();
        let mut ready = false;
        let mut last_probe = Instant::now() - Duration::from_secs(3);
        let mut misses = 0;
        loop {
            if cancel.load(Ordering::SeqCst) {
                terminate(&mut child);
                self.log(&c.id, "服务已停止，进程组已清理");
                return Ok(());
            }
            if let Some(exit) = match child.try_wait() {
                Ok(value) => value,
                Err(e) => {
                    terminate(&mut child);
                    return Err(err(e));
                }
            } {
                cleanup_group(child.id());
                return Err(ports::exit_error(
                    c.port, exit, ready, &log_path, log_offset,
                ));
            }
            if !ready && beginning.elapsed() > Duration::from_secs(300) {
                terminate(&mut child);
                return Err("启动超过 300 秒，未通过服务检查。请查看日志。".into());
            }
            if last_probe.elapsed() > Duration::from_secs(if ready { 5 } else { 1 }) {
                last_probe = Instant::now();
                if probe(c, ready).is_ok() && owns_listener(child.id(), c.port) {
                    misses = 0;
                    if !ready {
                        ready = true;
                        self.update(&c.id, |e| {
                            e.view.status = "running".into();
                            e.view.started_at = Some(now());
                            e.view.stage.clear();
                        });
                        self.log(&c.id, "服务检查通过");
                    } else {
                        self.update(&c.id, |e| {
                            e.view.status = "running".into();
                            e.view.error = None;
                        });
                    }
                } else if ready {
                    misses += 1;
                    if misses >= 3 {
                        self.update(&c.id, |e| {
                            e.view.status = "unhealthy".into();
                            e.view.error = Some("进程仍在运行，但服务检查连续失败".into());
                        });
                    }
                }
            }
            thread::sleep(Duration::from_millis(200));
        }
    }
    pub fn stop(&self, id: &str) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        let e = inner.get_mut(id).ok_or("找不到服务")?;
        if !e.busy {
            return Ok(());
        }
        e.cancel.store(true, Ordering::SeqCst);
        e.view.status = "stopping".into();
        e.view.stage = "正在中断并清理进程".into();
        Ok(())
    }
    pub fn shutdown(&self) {
        self.closing.store(true, Ordering::SeqCst);
        for e in self.inner.lock().unwrap().values() {
            e.cancel.store(true, Ordering::SeqCst);
        }
        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline && self.inner.lock().unwrap().values().any(|e| e.busy) {
            thread::sleep(Duration::from_millis(100));
        }
    }
}

fn client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(2))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(err)
}
fn probe(c: &Config, already_ready: bool) -> Result<()> {
    if c.protocol == "mcp-http" && already_ready {
        std::net::TcpStream::connect_timeout(
            &SocketAddrV4::new(Ipv4Addr::LOCALHOST, c.port).into(),
            Duration::from_secs(1),
        )
        .map_err(err)?;
        return Ok(());
    }
    let http = client()?;
    let base = format!("http://127.0.0.1:{}", c.port);
    if c.protocol == "mcp-http" {
        let response=http.post(format!("{base}{}",c.health_path)).header("Accept","application/json, text/event-stream")
          .json(&json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"launcher-probe","version":"0.1.0"}}})).send().map_err(err)?;
        let status = response.status();
        let session = response.headers().get("mcp-session-id").cloned();
        let is_sse = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .starts_with("text/event-stream");
        let value: Value = if is_sse {
            let mut reader = BufReader::new(response.take(1024 * 1024));
            let mut data = String::new();
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).map_err(err)? == 0 {
                    return Err("MCP 流结束但未返回 initialize 响应".into());
                }
                if line.trim().is_empty() {
                    if !data.is_empty() {
                        let v: Value = serde_json::from_str(&data).map_err(err)?;
                        if v["id"] == 1 {
                            break v;
                        }
                        data.clear();
                    }
                } else if let Some(part) = line.strip_prefix("data:") {
                    if !data.is_empty() {
                        data.push('\n');
                    }
                    data.push_str(part.trim());
                }
            }
        } else {
            serde_json::from_reader(response.take(1024 * 1024)).map_err(err)?
        };
        let version = value["result"]["protocolVersion"].as_str().unwrap_or("");
        let valid = status.is_success()
            && value["jsonrpc"] == "2.0"
            && value["id"] == 1
            && version == "2025-03-26"
            && value["result"]["capabilities"].is_object();
        let initialized = if valid {
            let mut request = http
                .post(format!("{base}{}", c.health_path))
                .header("Accept", "application/json, text/event-stream")
                .header("MCP-Protocol-Version", version)
                .json(&json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
            if let Some(ref s) = session {
                request = request.header("Mcp-Session-Id", s);
            }
            request
                .send()
                .map(|r| r.status().is_success())
                .unwrap_or(false)
        } else {
            false
        };
        if let Some(session) = session {
            let _ = http
                .delete(format!("{base}{}", c.health_path))
                .header("Mcp-Session-Id", session)
                .header("MCP-Protocol-Version", version)
                .send();
        }
        if !valid || !initialized {
            return Err("MCP 初始化失败（此版本探测支持 2025-03-26）".into());
        }
        return Ok(());
    }
    let response = http
        .get(format!(
            "{base}{}",
            if c.health_path.is_empty() {
                "/"
            } else {
                &c.health_path
            }
        ))
        .send()
        .map_err(err)?;
    if !response.status().is_success() {
        return Err("HTTP 检查失败".into());
    }
    if c.recipe == "mineru" {
        let body: Value = response.json().map_err(err)?;
        let feature = if c.version.starts_with("4.0.") {
            "output_formats"
        } else {
            "task_output_formats"
        };
        if body["status"] != "ok"
            || body["version"] != c.version
            || !body["features"][feature].is_array()
        {
            return Err(format!("响应不是预期的 MinerU {} API", c.version));
        }
        let models: Value = http
            .get(format!("{base}/v1/models"))
            .send()
            .map_err(err)?
            .json()
            .map_err(err)?;
        if !models["data"]
            .as_array()
            .map(|a| a.iter().any(|m| m["id"] == format!("mineru-{}", c.version)))
            .unwrap_or(false)
        {
            return Err("MinerU 版本与配方不符".into());
        }
    }
    Ok(())
}
#[cfg(unix)]
fn signal_group(pid: u32, signal: i32) {
    unsafe {
        libc::kill(-(pid as i32), signal);
    }
}
fn cleanup_group(pid: u32) {
    #[cfg(unix)]
    signal_group(pid, libc::SIGKILL);
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .output();
    }
}
fn terminate(child: &mut Child) {
    #[cfg(unix)]
    {
        signal_group(child.id(), libc::SIGINT);
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if child.try_wait().ok().flatten().is_some() {
                cleanup_group(child.id());
                return;
            }
            thread::sleep(Duration::from_millis(100));
        }
        signal_group(child.id(), libc::SIGTERM);
        thread::sleep(Duration::from_millis(500));
        cleanup_group(child.id());
    }
    #[cfg(windows)]
    cleanup_group(child.id());
    let _ = child.kill();
    let _ = child.wait();
}
fn owns_listener(pid: u32, port: u16) -> bool {
    #[cfg(target_os = "macos")]
    {
        let Ok(out) = Command::new("/usr/sbin/lsof")
            .args(["-nP", "-t", &format!("-iTCP:{port}"), "-sTCP:LISTEN"])
            .output()
        else {
            return false;
        };
        for listener in String::from_utf8_lossy(&out.stdout).lines() {
            if listener == pid.to_string() {
                return true;
            }
            if let Ok(p) = Command::new("/bin/ps")
                .args(["-p", listener, "-o", "pgid="])
                .output()
            {
                if String::from_utf8_lossy(&p.stdout).trim() == pid.to_string() {
                    return true;
                }
            }
        }
        false
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (pid, port);
        true
    }
}
