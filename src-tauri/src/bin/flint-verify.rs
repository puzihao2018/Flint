//! Headless integration harness. Does not connect to, or control, a running GUI.
use launcher_core::engine::Engine;
use std::{
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let arg = |key: &str| {
        args.iter()
            .position(|x| x == key)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let root =
        PathBuf::from(arg("--data-root").ok_or("--data-root required (GUI must be closed)")?);
    let engine = Engine::new(
        root.clone(),
        PathBuf::from(arg("--uv").ok_or("--uv required")?),
    )?;
    let id = if let Some(version) = arg("--version") {
        engine.add_release("mineru", &version)?
    } else {
        "mineru".into()
    };
    let mut config = engine.view(&id)?.config;
    config.port = arg("--port").unwrap_or("18371".into()).parse()?;
    config.tier = arg("--tier").unwrap_or("standard".into());
    engine.upsert(config)?;
    let mode = arg("--mode").unwrap_or("install".into());
    if mode == "install" {
        engine.install(&id)?;
    } else if mode == "serve" {
        engine.start(&id)?;
    } else {
        return Err("mode must be install or serve".into());
    }
    let deadline = Instant::now() + Duration::from_secs(7200);
    let mut previous = String::new();
    loop {
        let v = engine.view(&id)?;
        let fingerprint = format!("{}:{}", v.status, v.stage);
        if previous != fingerprint {
            println!("{}", serde_json::to_string(&v)?);
            previous = fingerprint;
        }
        if v.status == "error" {
            engine.shutdown();
            return Err(v.error.unwrap_or_default().into());
        }
        if mode == "install" && v.status == "stopped" {
            if v.config.models_ready {
                break;
            } else {
                return Err("installation did not complete".into());
            }
        }
        if root.join("STOP-VERIFIER").exists() || Instant::now() > deadline {
            engine.shutdown();
            let _ = std::fs::remove_file(root.join("STOP-VERIFIER"));
            break;
        }
        thread::sleep(Duration::from_millis(250));
    }
    Ok(())
}
