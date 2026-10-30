//! Offline validation only: never creates an Engine, installs, or launches a service.
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 || args[1] != "check" {
        eprintln!("Usage: flint-manifest check /path/to/launcher.yaml");
        std::process::exit(2);
    }
    match launcher_core::manifest::read(&args[2]) {
        Ok(c) => println!(
            "{}",
            serde_json::json!({
                "valid": true, "appId": c.app_id, "version": c.version,
                "name": c.name, "protocol": c.protocol, "installSteps": c.install_steps.len()
            })
        ),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
