//! Argument boundaries are preserved; structured arguments are never shell source.
use super::{Config, Engine, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path, process::Command};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandSpec {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}
impl CommandSpec {
    pub fn validate(&self) -> Result<()> {
        if self.program.trim().is_empty()
            || self.program.contains(['\0', '\n', '\r'])
            || self.program.contains("${")
        {
            return Err("program 必须是可执行文件名称或路径，不接受占位符".into());
        }
        for (key, value) in &self.env {
            if key.is_empty()
                || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                || key.as_bytes()[0].is_ascii_digit()
                || key.to_ascii_uppercase().starts_with("LAUNCHER_")
            {
                return Err("env 名称无效，且不能覆盖 LAUNCHER_ 保留变量".into());
            }
            validate_template(value)?;
        }
        for arg in &self.args {
            validate_template(arg)?;
        }
        Ok(())
    }
    pub fn preview(&self) -> String {
        // JSON is a display format, not an executable shell command.
        serde_json::to_string_pretty(self).unwrap()
    }
}
fn validate_template(value: &str) -> Result<()> {
    if value.contains('\0') {
        return Err("命令参数不能包含 NUL".into());
    }
    let mut rest = value;
    while let Some((_, tail)) = rest.split_once("${") {
        let (key, suffix) = tail.split_once('}').ok_or("占位符缺少结束括号")?;
        if !["port", "host", "dataDir", "cacheDir"].contains(&key) {
            return Err(format!("不支持的占位符：${{{key}}}"));
        }
        rest = suffix;
    }
    Ok(())
}
impl Engine {
    pub(super) fn structured_command(&self, c: &Config, spec: &CommandSpec) -> Result<Command> {
        spec.validate()?;
        let expand = |value: &str| {
            // Single pass: literal replacement values are never interpreted again.
            let mut output = String::new();
            let mut rest = value;
            while let Some((prefix, tail)) = rest.split_once("${") {
                let (key, suffix) = tail.split_once('}').unwrap();
                output.push_str(prefix);
                output.push_str(&match key {
                    "port" => c.port.to_string(),
                    "host" => "127.0.0.1".into(),
                    "dataDir" => self.instance_dir(&c.id).join("data").display().to_string(),
                    "cacheDir" => self.root.join("cache").join(&c.id).display().to_string(),
                    _ => unreachable!(),
                });
                rest = suffix;
            }
            output.push_str(rest);
            output
        };
        let args: Vec<_> = spec.args.iter().map(|v| expand(v)).collect();
        // Keep the same login-shell runtime lookup as legacy commands (nvm, uv,
        // Homebrew), but pass program and args as positional data, never source.
        #[cfg(unix)]
        let mut cmd = {
            let shell = if Path::new("/bin/zsh").exists() {
                "/bin/zsh"
            } else {
                "/bin/sh"
            };
            let mut cmd = Command::new(shell);
            cmd.args(["-lc", "exec \"$@\"", "flint-run", &spec.program]);
            cmd
        };
        #[cfg(not(unix))]
        let mut cmd = Command::new(&spec.program);
        cmd.args(args).current_dir(&c.cwd);
        cmd.envs(spec.env.iter().map(|(k, v)| (k, expand(v))));
        Ok(cmd)
    }
}
