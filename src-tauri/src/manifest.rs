//! Explicit, narrow import format. Import only prepares a configuration; it never runs code.
use crate::engine::{CommandSpec, Config, Result};
use serde::Deserialize;
use std::path::Path;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    schema_version: String,
    name: String,
    category: String,
    environment: Environment,
    parameters: Parameters,
    service: Service,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Environment {
    mode: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters {
    port: Port,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Port {
    default: u16,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Service {
    protocol: String,
    cwd: String,
    command: String,
    health_path: String,
}
pub fn parse(text: &str, base: &Path) -> Result<Config> {
    if text.len() > 65536 {
        return Err("清单不得超过 64 KB".into());
    }
    let value: serde_yaml::Value =
        serde_yaml::from_str(text).map_err(|e| format!("清单格式不受支持：{e}"))?;
    match value.get("schemaVersion").and_then(|v| v.as_str()) {
        Some("0.2") => return parse_v2(text, base),
        Some("0.1") => (),
        _ => return Err("不支持的 schemaVersion；请使用字符串 0.1 或 0.2".into()),
    }
    let m: Manifest = serde_yaml::from_str(text).map_err(|e| format!("清单格式不受支持：{e}"))?;
    if m.schema_version != "0.1" || m.category != "service" || m.environment.mode != "existing" {
        return Err("本版导入仅支持 schemaVersion 0.1、category service、environment.mode existing；受管安装目前使用内置 MinerU 配方。".into());
    }
    let mut c = Config::mineru();
    c.id.clear();
    c.recipe = "custom".into();
    c.version = String::new();
    c.name = m.name;
    c.port = m.parameters.port.default;
    c.command = m.service.command;
    c.protocol = m.service.protocol;
    c.health_path = m.service.health_path;
    c.cwd = base
        .join(m.service.cwd)
        .canonicalize()
        .map_err(|e| format!("工作目录不存在：{e}"))?
        .display()
        .to_string();
    c.installed = true;
    c.models_ready = true;
    c.validate()?;
    Ok(c)
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ManifestV2 {
    schema_version: String,
    id: String,
    version: String,
    name: String,
    environment: Environment,
    parameters: Parameters,
    #[serde(default)]
    install: Vec<CommandSpec>,
    service: ServiceV2,
    #[serde(default)]
    platforms: Vec<String>,
    // Reserved metadata for tooling; never executed or interpreted by Flint.
    #[serde(default)]
    extensions: std::collections::BTreeMap<String, serde_json::Value>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ServiceV2 {
    protocol: String,
    #[serde(default = "project_root")]
    cwd: String,
    run: CommandSpec,
    health_path: Option<String>,
}
fn project_root() -> String {
    ".".into()
}
fn parse_v2(text: &str, base: &Path) -> Result<Config> {
    let m: ManifestV2 = serde_yaml::from_str(text).map_err(|e| format!("清单格式不受支持：{e}"))?;
    if m.schema_version != "0.2" || m.environment.mode != "existing" {
        return Err("0.2 仅支持 environment.mode: existing；通用 managed 环境尚未实现".into());
    }
    if m.id.is_empty()
        || m.id.len() > 120
        || !m.id.as_bytes()[0].is_ascii_lowercase()
        || !m
            .id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || ".-_".contains(c))
    {
        return Err(
            "应用 id 需以小写字母开头，只含小写字母、数字、点、横线或下划线，最多 120 字符".into(),
        );
    }
    if m.version.trim().is_empty()
        || m.version.len() > 80
        || m.version.chars().any(char::is_control)
    {
        return Err("version 需是非空字符串，最多 80 字符".into());
    }
    if m.platforms
        .iter()
        .any(|p| !["macos", "linux", "windows"].contains(&p.as_str()))
    {
        return Err("platforms 仅接受 macos、linux、windows".into());
    }
    if !m.platforms.is_empty() && !m.platforms.iter().any(|p| p == std::env::consts::OS) {
        return Err("此清单未声明支持当前平台".into());
    }
    let _ = m.extensions;
    let mut c = Config::mineru();
    c.id.clear();
    c.app_id = Some(m.id);
    c.version = m.version;
    c.recipe = "custom".into();
    c.name = m.name;
    c.port = m.parameters.port.default;
    c.cwd = base
        .join(m.service.cwd)
        .canonicalize()
        .map_err(|e| format!("工作目录不存在：{e}"))?
        .display()
        .to_string();
    c.run = Some(m.service.run);
    c.install_steps = m.install;
    c.protocol = m.service.protocol;
    c.health_path = match m.service.health_path {
        Some(path) if !path.is_empty() => path,
        _ if c.protocol == "mcp-http" => {
            return Err("MCP 服务需要显式声明 healthPath，例如 /mcp".into())
        }
        _ => "/".into(),
    };
    c.installed = c.install_steps.is_empty();
    c.models_ready = true;
    c.validate()?;
    Ok(c)
}
pub fn read(path: &str) -> Result<Config> {
    let path = Path::new(path);
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 65536 {
        return Err("清单不得超过 64 KB".into());
    }
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    parse(&text, path.parent().ok_or("清单目录不存在")?)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn import_rejects_install_and_resolves_cwd() {
        let root = tempfile::tempdir().unwrap();
        let text="schemaVersion: '0.1'\nname: Demo\ncategory: service\nenvironment: {mode: existing}\nparameters: {port: {default: 3000}}\nservice:\n  protocol: http-api\n  cwd: .\n  command: uv run server --port ${port}\n  healthPath: /healthz\n";
        let c = parse(text, root.path()).unwrap();
        assert!(Path::new(&c.cwd).is_absolute());
        assert!(c.command.contains("${port}"));
        assert!(parse(&(text.to_owned() + "install: []\n"), root.path()).is_err());
        assert!(parse(&text.replace("existing", "managed"), root.path()).is_err());
    }
}

#[cfg(test)]
mod v2_tests {
    use super::*;
    fn demo() -> serde_json::Value {
        serde_json::json!({"schemaVersion":"0.2","id":"example.test","version":"1.0.0","name":"Demo",
            "environment":{"mode":"existing"},"parameters":{"port":{"default":3000}},
            "service":{"protocol":"http-api","run":{"program":"node","args":["server.mjs"]}}})
    }
    #[test]
    fn v2_defaults_and_rejections() {
        let root = tempfile::tempdir().unwrap();
        let parse_value = |v: serde_json::Value| parse(&v.to_string(), root.path());
        let config = parse_value(demo()).unwrap();
        assert_eq!(config.health_path, "/");
        assert!(config.installed);
        let mut v = demo();
        v["service"]["protocol"] = "mcp-http".into();
        assert!(parse_value(v.clone()).is_err());
        v["service"]["healthPath"] = "/mcp".into();
        assert!(parse_value(v).is_ok());
        for (field, value) in [("schemaVersion", "9"), ("id", "../bad"), ("version", "")] {
            let mut v = demo();
            v[field] = value.into();
            assert!(parse_value(v).is_err());
        }
        let mut v = demo();
        v["environment"]["mode"] = "managed".into();
        assert!(parse_value(v).is_err());
        let mut v = demo();
        v["instal"] = serde_json::json!([]);
        assert!(parse_value(v).is_err());
        let mut v = demo();
        v["platforms"] = serde_json::json!(["unknown"]);
        assert!(parse_value(v).is_err());
        for env in [
            serde_json::json!({"LAUNCHER_PORT":"99"}),
            serde_json::json!({"PORT":"${unknown}"}),
        ] {
            let mut v = demo();
            v["service"]["run"]["env"] = env;
            assert!(parse_value(v).is_err());
        }
        let mut v = demo();
        v["service"]["run"]["arguments"] = serde_json::json!([]);
        assert!(parse_value(v).is_err());
        let mut v = demo();
        v["service"]["healthPath"] = "//external/path".into();
        assert!(parse_value(v).is_err());
        let mut v = demo();
        v["parameters"]["port"]["default"] = 80.into();
        assert!(parse_value(v).is_err());
        let mut v = demo();
        v["extensions"] = serde_json::json!({"example":{"tag":"demo"}});
        assert!(parse_value(v).is_ok());
    }
    #[test]
    fn checked_in_examples_parse() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples");
        for name in ["http-service", "node-webapp", "python-webapp"] {
            let c = read(base.join(name).join("launcher.yaml").to_str().unwrap()).unwrap();
            assert!(Path::new(&c.cwd).is_dir());
            assert_eq!(c.install_steps.is_empty(), name == "http-service");
        }
    }
}
