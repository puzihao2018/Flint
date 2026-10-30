use super::*;

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemovalOptions {
    pub environment: bool,
    pub models: bool,
    pub data: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovalItem {
    key: &'static str,
    path: String,
    bytes: u64,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovalPreview {
    managed: bool,
    shared: bool,
    items: Vec<RemovalItem>,
}

impl Engine {
    fn removal_paths(&self, c: &Config) -> Vec<(&'static str, PathBuf)> {
        let mut paths = Vec::new();
        if c.recipe == "mineru" {
            paths.push((
                "environment",
                self.env_dir(c).parent().unwrap().to_path_buf(),
            ));
            paths.push(("models", self.model_dir(c)));
        }
        paths.push(("data", self.instance_dir(&c.id)));
        // These directories are shared download caches, never instance data.
        if !["uv", "huggingface", "modelscope"].contains(&c.id.as_str()) {
            paths.push(("data", self.root.join("cache").join(&c.id)));
        }
        paths
    }
    pub(super) fn model_dir(&self, c: &Config) -> PathBuf {
        if c.version == "4.1.2" {
            self.root.join("models/mineru")
        } else {
            self.root
                .join("models")
                .join(format!("mineru-{}", c.version))
        }
    }
    fn shared_release(&self, c: &Config) -> bool {
        self.inner.lock().unwrap().values().any(|e| {
            e.view.config.id != c.id
                && e.view.config.recipe == c.recipe
                && e.view.config.version == c.version
        })
    }
    fn check_removal_options(&self, c: &Config, options: &RemovalOptions) -> Result<()> {
        c.validate()?;
        if options.environment || options.models {
            if c.recipe != "mineru" {
                return Err("自定义服务的项目与环境不由 Flint 安装，不能卸载".into());
            }
            if self.shared_release(c) {
                return Err("其他服务仍共用此版本，不能卸载其环境或模型".into());
            }
        }
        Ok(())
    }
    pub fn removal_preview(&self, id: &str) -> Result<RemovalPreview> {
        let c = self.view(id)?.config;
        c.validate()?;
        let items = self
            .removal_paths(&c)
            .into_iter()
            .map(|(key, path)| {
                validate_path(&self.root, &path)?;
                Ok(RemovalItem {
                    key,
                    bytes: tree_bytes(&path)?,
                    path: path.display().to_string(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(RemovalPreview {
            managed: c.recipe == "mineru",
            shared: c.recipe == "mineru" && self.shared_release(&c),
            items,
        })
    }
    /// Stops owned work first, then deletes only explicitly selected managed paths.
    pub fn remove_with_options(&self, id: &str, options: RemovalOptions) -> Result<()> {
        let c = {
            let mut inner = self.inner.lock().unwrap();
            let e = inner.get_mut(id).ok_or("找不到服务")?;
            if e.removing {
                return Err("服务正在删除".into());
            }
            if e.view.status == "interrupted" {
                return Err("请先确认旧进程已结束并清除中断记录".into());
            }
            e.removing = true;
            e.view.error = None;
            if e.busy {
                e.cancel.store(true, Ordering::SeqCst);
                e.view.status = "stopping".into();
                e.view.stage = "正在停止服务，以便删除".into();
            }
            e.view.config.clone()
        };
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let mut inner = self.inner.lock().unwrap();
            let e = inner.get_mut(id).ok_or("找不到服务")?;
            if !e.busy {
                e.busy = true;
                e.view.status = "removing".into();
                e.view.stage = "正在删除所选内容".into();
                break;
            }
            if Instant::now() >= deadline {
                e.removing = false;
                return Err("服务尚未停止，未删除文件；请稍后重试".into());
            }
            drop(inner);
            thread::sleep(Duration::from_millis(100));
        }
        let result = self.remove_reserved(&c, &options);
        if let Err(ref error) = result {
            self.update(id, |e| {
                e.busy = false;
                e.removing = false;
                e.view.status = "error".into();
                e.view.stage.clear();
                e.view.error = Some(format!("删除未完成：{error}。服务登记已保留，可重试。"));
            });
        }
        result
    }
    fn remove_reserved(&self, c: &Config, options: &RemovalOptions) -> Result<()> {
        self.check_removal_options(c, options)?;
        let paths: Vec<_> = self
            .removal_paths(c)
            .into_iter()
            .filter(|(key, _)| match *key {
                "environment" => options.environment,
                "models" => options.models,
                _ => options.data,
            })
            .collect();
        // Validate the entire plan before touching files. Symlink ancestors are
        // rejected; a symlink at the selected leaf is removed without following it.
        for (_, path) in &paths {
            validate_path(&self.root, path)?;
        }
        {
            let mut inner = self.inner.lock().unwrap();
            let e = inner.get_mut(&c.id).ok_or("找不到服务")?;
            let before = e.view.config.clone();
            if options.environment {
                e.view.config.installed = false;
            }
            if options.environment || options.models {
                e.view.config.models_ready = false;
            }
            if let Err(error) = self.save_entries(&inner) {
                inner.get_mut(&c.id).unwrap().view.config = before;
                return Err(error);
            }
        }
        for (_, path) in paths {
            validate_path(&self.root, &path)?;
            match fs::symlink_metadata(&path) {
                Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => {
                    fs::remove_dir_all(&path)
                }
                Ok(_) => fs::remove_file(&path),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e),
            }
            .map_err(|e| format!("{}: {e}", path.display()))?;
        }
        let mut inner = self.inner.lock().unwrap();
        let removed = inner.remove(&c.id).ok_or("找不到服务")?;
        if let Err(error) = self.save_entries(&inner) {
            inner.insert(c.id.clone(), removed);
            return Err(error);
        }
        Ok(())
    }
}

fn validate_path(root: &Path, path: &Path) -> Result<()> {
    use std::path::Component;
    let relative = path
        .strip_prefix(root)
        .map_err(|_| "删除路径不属于 Flint 数据目录")?;
    let components: Vec<_> = relative.components().collect();
    if components.is_empty()
        || components
            .iter()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("无效的删除路径".into());
    }
    let mut parent = root.to_path_buf();
    for component in &components[..components.len() - 1] {
        parent.push(component);
        match fs::symlink_metadata(&parent) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err("删除路径的父目录是符号链接，请先检查存储位置".into())
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => break,
            Err(e) => return Err(err(e)),
        }
    }
    Ok(())
}
fn tree_bytes(path: &Path) -> Result<u64> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(err(e)),
    };
    if meta.file_type().is_symlink() {
        return Ok(0);
    }
    if meta.is_file() {
        return Ok(meta.len());
    }
    let mut total = 0_u64;
    for entry in fs::read_dir(path).map_err(err)? {
        total = total.saturating_add(tree_bytes(&entry.map_err(err)?.path())?);
    }
    Ok(total)
}
