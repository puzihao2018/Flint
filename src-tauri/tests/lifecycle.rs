use launcher_core::engine::{ensure_port_free, Config, Engine};
use std::{
    fs,
    net::TcpListener,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};
fn port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}
fn engine() -> (tempfile::TempDir, Engine) {
    let root = tempfile::tempdir().unwrap();
    let e = Engine::new(root.path().to_path_buf(), PathBuf::from("/missing-uv")).unwrap();
    (root, e)
}
fn add(e: &Engine, dir: &std::path::Path, mode: &str) -> String {
    let mut c = Config::mineru();
    c.id.clear();
    c.name = format!("Fixture {mode}");
    c.recipe = "custom".into();
    c.cwd = dir.display().to_string();
    c.port = port();
    c.health_path = "/health".into();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/service.py");
    c.command = format!(
        "/usr/bin/python3 '{}' --port ${{port}} --mode {mode}",
        fixture.display()
    );
    if mode.starts_with("mcp") {
        c.protocol = "mcp-http".into();
        c.health_path = "/mcp".into();
    }
    e.upsert(c).unwrap()
}
fn wait(e: &Engine, id: &str, status: &str) {
    let deadline = Instant::now() + Duration::from_secs(18);
    while Instant::now() < deadline {
        if e.view(id).unwrap().status == status {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!(
        "wanted {status}, got {:?}\n{}",
        e.view(id),
        e.logs(id).unwrap()
    );
}
#[test]
fn http_lifecycle_config_and_port_conflict() {
    let (root, e) = engine();
    let id = add(&e, root.path(), "http");
    let c = e.view(&id).unwrap().config;
    let listener = TcpListener::bind(("127.0.0.1", c.port)).unwrap();
    assert!(e.start(&id).unwrap_err().contains("已被占用"));
    drop(listener);
    e.start(&id).unwrap();
    for _ in 0..30 {
        assert!(e.start(&id).is_err());
    }
    wait(&e, &id, "running");
    assert!(ensure_port_free(c.port).is_err());
    assert!(e.upsert(c.clone()).is_err());
    assert!(e.remove(&id).is_err());
    e.stop(&id).unwrap();
    wait(&e, &id, "stopped");
    ensure_port_free(c.port).unwrap();
    let loaded = Engine::new(root.path().to_path_buf(), PathBuf::from("/missing-uv")).unwrap();
    assert_eq!(loaded.view(&id).unwrap().config.port, c.port);
    e.start(&id).unwrap();
    wait(&e, &id, "running");
    e.shutdown();
    ensure_port_free(c.port).unwrap();
    assert!(e.start(&id).is_err());
}
#[test]
fn descendants_are_stopped() {
    let (root, e) = engine();
    let id = add(&e, root.path(), "tree");
    e.start(&id).unwrap();
    wait(&e, &id, "running");
    let pid = fs::read_to_string(root.path().join("child.pid"))
        .unwrap()
        .parse::<i32>()
        .unwrap();
    e.stop(&id).unwrap();
    wait(&e, &id, "stopped");
    ensure_port_free(e.view(&id).unwrap().config.port).unwrap();
    thread::sleep(Duration::from_millis(300));
    assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
}
#[test]
fn crash_and_wrong_health_are_not_running() {
    let (root, e) = engine();
    let id = add(&e, root.path(), "crash");
    e.start(&id).unwrap();
    wait(&e, &id, "error");
    let id = add(&e, root.path(), "bad-health");
    e.start(&id).unwrap();
    thread::sleep(Duration::from_secs(3));
    assert_eq!(e.view(&id).unwrap().status, "starting");
    e.stop(&id).unwrap();
    wait(&e, &id, "stopped");
}
#[test]
fn interrupted_record_blocks_restart_and_preserves_config() {
    let (root, e) = engine();
    let id = add(&e, root.path(), "http");
    fs::create_dir_all(root.path().join("instances").join(&id)).unwrap();
    fs::write(
        root.path().join("instances").join(&id).join("active.json"),
        "{}",
    )
    .unwrap();
    let loaded = Engine::new(root.path().to_path_buf(), PathBuf::from("none")).unwrap();
    assert_eq!(loaded.view(&id).unwrap().status, "interrupted");
    assert!(loaded.start(&id).is_err());
    assert!(loaded.remove(&id).is_err());
    loaded.acknowledge(&id).unwrap();
    assert_eq!(loaded.view(&id).unwrap().status, "stopped");
}

#[test]
fn removing_releases_preserves_files_other_services_and_empty_registry() {
    let (root, e) = engine();
    let id = e.add_release("mineru", "4.0.12").unwrap();
    let other = e.view("mineru").unwrap().config;
    let retained = [
        "apps/mineru/4.0.12/env/retained.txt",
        "models/mineru-4.0.12/retained.txt",
        "instances/mineru-4-0-12/service.log",
    ];
    for relative in retained {
        let path = root.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "keep").unwrap();
    }
    e.remove(&id).unwrap();
    assert!(e.view(&id).is_err());
    let loaded = Engine::new(root.path().to_path_buf(), "/missing-uv".into()).unwrap();
    assert!(loaded.view(&id).is_err());
    assert_eq!(loaded.view("mineru").unwrap().config.port, other.port);
    for relative in retained {
        assert_eq!(
            fs::read_to_string(root.path().join(relative)).unwrap(),
            "keep"
        );
    }
    // A removed release remains available in the market.
    let restored = loaded.add_release("mineru", "4.0.12").unwrap();
    loaded.remove(&restored).unwrap();
    loaded.remove("mineru").unwrap();
    let empty = Engine::new(root.path().to_path_buf(), "/missing-uv".into()).unwrap();
    assert!(empty.views().is_empty());
}

#[test]
fn removal_save_failure_keeps_registration_and_custom_project() {
    let (root, e) = engine();
    let id = add(&e, root.path(), "http");
    let project = root.path().join("user-project.txt");
    fs::write(&project, "keep").unwrap();
    let previous = fs::read(root.path().join("services.json")).unwrap();
    let blocked = root.path().join("services.json.tmp");
    fs::create_dir(&blocked).unwrap();
    assert!(e.remove(&id).is_err());
    assert!(e.view(&id).is_ok());
    assert_eq!(
        fs::read(root.path().join("services.json")).unwrap(),
        previous
    );
    fs::remove_dir(blocked).unwrap();
    e.remove(&id).unwrap();
    assert!(e.view(&id).is_err());
    assert_eq!(fs::read_to_string(project).unwrap(), "keep");
}
#[test]
fn mcp_json_and_sse_initialize_then_release_session() {
    for mode in ["mcp-json", "mcp-sse"] {
        let (root, e) = engine();
        let id = add(&e, root.path(), mode);
        e.start(&id).unwrap();
        wait(&e, &id, "running");
        assert!(root.path().join("initialized").exists());
        assert!(root.path().join("deleted").exists());
        e.shutdown();
    }
}
#[test]
fn missing_installer_is_reported() {
    let (_root, e) = engine();
    e.install("mineru").unwrap();
    wait(&e, "mineru", "error");
    assert!(!e.view("mineru").unwrap().config.installed);
}
#[test]
fn invalid_and_corrupt_configuration_is_rejected() {
    let (root, e) = engine();
    let mut c = Config::mineru();
    c.id.clear();
    c.command = "echo hi".into();
    c.cwd = root.path().display().to_string();
    c.health_path = "//example.com".into();
    assert!(e.upsert(c).is_err());
    let path = root.path().join("services.json");
    fs::write(&path, "invalid").unwrap();
    assert!(Engine::new(root.path().to_path_buf(), PathBuf::from("none")).is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), "invalid");
}

#[cfg(unix)]
#[test]
fn install_cancellation_cleans_its_process_group() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let fake = root.path().join("fake-uv");
    fs::write(&fake, "#!/bin/sh\nsleep 60\n").unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    let e = Engine::new(root.path().join("data"), fake).unwrap();
    e.install("mineru").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while e.view("mineru").unwrap().pid.is_none() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(50));
    }
    let pid = e.view("mineru").unwrap().pid.unwrap();
    e.stop("mineru").unwrap();
    wait(&e, "mineru", "stopped");
    assert!(!e.view("mineru").unwrap().config.installed);
    assert_eq!(unsafe { libc::kill(-(pid as i32), 0) }, -1);
}

#[test]
fn releases_coexist_and_legacy_config_migrates_without_overwrite() {
    let root = tempfile::tempdir().unwrap();
    let mut legacy = serde_json::to_value(Config::mineru()).unwrap();
    legacy.as_object_mut().unwrap().remove("version");
    legacy["port"] = 8341.into();
    legacy["installed"] = true.into();
    legacy["modelsReady"] = true.into();
    fs::write(
        root.path().join("services.json"),
        serde_json::to_vec(&vec![legacy]).unwrap(),
    )
    .unwrap();
    let e = Engine::new(root.path().to_path_buf(), "/missing-uv".into()).unwrap();
    let old = e.view("mineru").unwrap().config;
    assert_eq!(old.version, "4.1.2");
    assert_eq!(old.port, 8341);
    assert!(old.models_ready);
    let id = e.add_release("mineru", "4.0.12").unwrap();
    assert_eq!(e.add_release("mineru", "4.0.12").unwrap(), id);
    assert_eq!(e.add_release("mineru", "4.1.2").unwrap(), "mineru");
    assert_eq!(e.views().len(), 2);
    let mut new = e.view(&id).unwrap().config;
    assert_eq!(new.version, "4.0.12");
    assert!(!new.installed);
    assert!(!new.models_ready);
    assert_ne!(new.port, old.port);
    assert!(e.start(&id).is_err());
    new.version = "4.1.2".into();
    assert!(e.upsert(new).is_err());
    assert!(e.add_release("mineru", "../../invalid").is_err());
    assert!(e.add_release("unknown", "4.0.12").is_err());
    let loaded = Engine::new(root.path().to_path_buf(), "/missing-uv".into()).unwrap();
    assert_eq!(loaded.view(&id).unwrap().config.version, "4.0.12");
    assert!(loaded.view("mineru").unwrap().config.models_ready);
}

fn uninstall_options(
    environment: bool,
    models: bool,
    data: bool,
) -> launcher_core::engine::RemovalOptions {
    launcher_core::engine::RemovalOptions {
        environment,
        models,
        data,
    }
}
fn fixture_file(root: &std::path::Path, relative: &str) -> PathBuf {
    let p = root.join(relative);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, "keep until selected").unwrap();
    p
}
#[test]
fn uninstall_is_per_version_and_preserves_shared_downloads() {
    let (root, e) = engine();
    let id = e.add_release("mineru", "4.0.12").unwrap();
    let env = fixture_file(root.path(), "apps/mineru/4.0.12/env/test");
    let model = fixture_file(root.path(), "models/mineru-4.0.12/test");
    let data = fixture_file(root.path(), "instances/mineru-4-0-12/data/test");
    let cache = fixture_file(root.path(), "cache/mineru-4-0-12/test");
    let kept: Vec<_> = [
        "apps/mineru/4.1.2/env/test",
        "models/mineru/test",
        "cache/uv/test",
        "cache/huggingface/test",
        "runtimes/python/test",
    ]
    .iter()
    .map(|p| fixture_file(root.path(), p))
    .collect();
    let preview = serde_json::to_value(e.removal_preview(&id).unwrap()).unwrap();
    assert_eq!(preview["managed"], true);
    assert!(preview["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|x| x["bytes"].as_u64().unwrap() > 0));
    e.remove_with_options(&id, uninstall_options(true, true, true))
        .unwrap();
    for p in [env, model, data, cache] {
        assert!(!p.exists());
    }
    for p in kept {
        assert!(p.exists());
    }
    assert!(e.view("mineru").is_ok());
    assert!(Engine::new(root.path().into(), "/missing".into())
        .unwrap()
        .view(&id)
        .is_err());
}
#[test]
fn uninstall_only_selected_categories_and_legacy_model_path() {
    let (root, e) = engine();
    let env = fixture_file(root.path(), "apps/mineru/4.1.2/env/test");
    let model = fixture_file(root.path(), "models/mineru/test");
    let data = fixture_file(root.path(), "instances/mineru/data/test");
    e.remove_with_options("mineru", uninstall_options(true, false, false))
        .unwrap();
    assert!(!env.exists());
    assert!(model.exists());
    assert!(data.exists());
    let id = e.add_release("mineru", "4.1.2").unwrap();
    e.remove_with_options(&id, uninstall_options(false, true, false))
        .unwrap();
    assert!(!model.exists());
    assert!(data.exists());
}
#[test]
fn removal_stops_owned_service_and_preserves_external_project() {
    let (root, e) = engine();
    let id = add(&e, root.path(), "http");
    let project = fixture_file(root.path(), "project/test");
    let port = e.view(&id).unwrap().config.port;
    e.start(&id).unwrap();
    wait(&e, &id, "running");
    e.remove_with_options(&id, uninstall_options(false, false, true))
        .unwrap();
    ensure_port_free(port).unwrap();
    assert!(e.view(&id).is_err());
    assert!(project.exists());
    assert!(!root.path().join("instances").join(id).exists());
}
#[test]
fn uninstall_refuses_shared_versions_and_custom_environments() {
    let root = tempfile::tempdir().unwrap();
    let first = Config::mineru();
    let mut second = first.clone();
    second.id = "duplicate".into();
    second.port = 8001;
    fs::write(
        root.path().join("services.json"),
        serde_json::to_vec(&vec![first, second]).unwrap(),
    )
    .unwrap();
    let e = Engine::new(root.path().into(), "/missing".into()).unwrap();
    let model = fixture_file(root.path(), "models/mineru/test");
    assert!(e
        .remove_with_options("mineru", uninstall_options(false, true, false))
        .is_err());
    assert!(model.exists());
    assert!(e.view("mineru").is_ok());
    let id = add(&e, root.path(), "http");
    assert!(e
        .remove_with_options(&id, uninstall_options(true, false, false))
        .is_err());
    assert!(e.view(&id).is_ok());
}
#[cfg(unix)]
#[test]
fn uninstall_never_follows_links_or_deletes_through_symlink_ancestors() {
    use std::os::unix::fs::symlink;
    let (root, e) = engine();
    let external = tempfile::tempdir().unwrap();
    let outside = fixture_file(external.path(), "external.txt");
    fs::create_dir_all(root.path().join("models")).unwrap();
    symlink(external.path(), root.path().join("models/mineru")).unwrap();
    e.remove_with_options("mineru", uninstall_options(false, true, false))
        .unwrap();
    assert!(outside.exists());
    assert!(!root.path().join("models/mineru").exists());
    let id = e.add_release("mineru", "4.0.12").unwrap();
    let env = fixture_file(root.path(), "apps/mineru/4.0.12/env/test");
    fs::remove_dir(root.path().join("models")).unwrap();
    symlink(external.path(), root.path().join("models")).unwrap();
    assert!(e
        .remove_with_options(&id, uninstall_options(true, true, false))
        .is_err());
    assert!(env.exists());
    assert!(outside.exists());
}
#[test]
fn uninstall_persistence_failure_happens_before_file_deletion() {
    let (root, e) = engine();
    let env = fixture_file(root.path(), "apps/mineru/4.1.2/env/test");
    fs::create_dir(root.path().join("services.json.tmp")).unwrap();
    assert!(e
        .remove_with_options("mineru", uninstall_options(true, true, true))
        .is_err());
    assert!(env.exists());
    assert!(e.view("mineru").is_ok());
    fs::remove_dir(root.path().join("services.json.tmp")).unwrap();
    e.remove_with_options("mineru", uninstall_options(true, true, true))
        .unwrap();
    assert!(!env.exists());
}

#[cfg(target_os = "macos")]
#[test]
fn port_preflight_detects_ipv4_and_ipv6_loopback_and_wildcard_listeners() {
    for address in ["127.0.0.1:0", "0.0.0.0:0", "[::1]:0", "[::]:0"] {
        let listener = TcpListener::bind(address).unwrap();
        let port = listener.local_addr().unwrap().port();
        let error = ensure_port_free(port).unwrap_err();
        assert!(error.contains(&format!("端口 {port} 已被占用")), "{error}");
        assert!(
            error.contains(&format!("PID {}", std::process::id())),
            "{error}"
        );
        assert!(
            error.contains(if address.starts_with('[') {
                "IPv6"
            } else {
                "IPv4"
            }),
            "{error}"
        );
        drop(listener);
        ensure_port_free(port).unwrap();
    }
}
#[test]
fn startup_collision_is_clear_and_old_errors_do_not_taint_later_failures() {
    let (root, e) = engine();
    let id = add(&e, root.path(), "address-in-use");
    e.start(&id).unwrap();
    wait(&e, &id, "error");
    let message = e.view(&id).unwrap().error.unwrap();
    assert!(message.contains("已被占用"), "{message}");
    assert!(message.contains("修改此服务的端口"), "{message}");
    let mut config = e.view(&id).unwrap().config;
    config.command = config.command.replace("address-in-use", "crash");
    e.upsert(config).unwrap();
    e.start(&id).unwrap();
    wait(&e, &id, "error");
    let message = e.view(&id).unwrap().error.unwrap();
    assert!(message.contains("服务意外退出"), "{message}");
    assert!(!message.contains("已被占用"), "{message}");
}

#[test]
fn structured_manifest_install_run_persist_and_reconfigure() {
    use launcher_core::manifest;
    let (root, e) = engine();
    let project = root.path().join("project with spaces");
    fs::create_dir(&project).unwrap();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/service.py");
    // These metacharacters must remain literal arguments, never shell source.
    let literal = "space ' quote ; $(touch SHOULD_NOT_EXIST) `touch ALSO_NOT`";
    let manifest = serde_json::json!({
        "schemaVersion":"0.2", "id":"example.test", "version":"1.0.0", "name":"Structured",
        "environment":{"mode":"existing"}, "parameters":{"port":{"default":port()}},
        "install":[{"program":"/usr/bin/python3", "args":["-c",
          "import os,sys,pathlib; pathlib.Path('arguments.txt').write_text(sys.argv[1]); pathlib.Path('port.txt').write_text(os.environ['TEST_PORT']); assert os.path.isdir(os.environ['LAUNCHER_CACHE_DIR'])", literal],
          "env":{"TEST_PORT":"${port}"}}],
        "service":{"protocol":"http-api", "healthPath":"/health", "run":{
          "program":"/usr/bin/python3", "args":[fixture.to_str().unwrap(),"--port","${port}","--mode","http"]}}
    });
    let c = manifest::parse(&manifest.to_string(), &project).unwrap();
    assert!(!project.join("arguments.txt").exists());
    let id = e.upsert(c).unwrap();
    assert!(!project.join("arguments.txt").exists());
    assert!(e.start(&id).unwrap_err().contains("安装"));
    e.install(&id).unwrap();
    wait(&e, &id, "stopped");
    assert!(e.view(&id).unwrap().config.installed);
    assert_eq!(
        fs::read_to_string(project.join("arguments.txt")).unwrap(),
        literal
    );
    assert!(!project.join("SHOULD_NOT_EXIST").exists());
    assert!(!project.join("ALSO_NOT").exists());
    let config = e.view(&id).unwrap().config;
    assert_eq!(
        fs::read_to_string(project.join("port.txt")).unwrap(),
        config.port.to_string()
    );
    e.start(&id).unwrap();
    wait(&e, &id, "running");
    e.stop(&id).unwrap();
    wait(&e, &id, "stopped");
    ensure_port_free(config.port).unwrap();
    let mut c = config.clone();
    c.port = port();
    e.upsert(c).unwrap();
    assert!(e.view(&id).unwrap().config.installed);
    e.start(&id).unwrap();
    wait(&e, &id, "running");
    e.stop(&id).unwrap();
    wait(&e, &id, "stopped");
    let loaded = Engine::new(root.path().to_path_buf(), PathBuf::from("/missing-uv")).unwrap();
    let mut c = loaded.view(&id).unwrap().config;
    assert_eq!(c.app_id.as_deref(), Some("example.test"));
    assert_eq!(c.run, config.run);
    assert!(c.installed);
    c.install_steps[0].args = vec!["-c".into(), "raise SystemExit(7)".into()];
    loaded.upsert(c).unwrap();
    assert!(!loaded.view(&id).unwrap().config.installed);
    loaded.install(&id).unwrap();
    wait(&loaded, &id, "error");
    assert!(!loaded.view(&id).unwrap().config.installed);
    assert!(loaded.start(&id).is_err());
    assert!(loaded
        .view(&id)
        .unwrap()
        .error
        .unwrap()
        .contains("安装步骤"));
}

#[test]
fn custom_install_cancellation_keeps_start_disabled() {
    let (root, e) = engine();
    let id = add(&e, root.path(), "http");
    let mut c = e.view(&id).unwrap().config;
    c.install_steps = vec![launcher_core::engine::CommandSpec {
        program: "/usr/bin/python3".into(),
        args: vec!["-c".into(), "import time; time.sleep(60)".into()],
        env: Default::default(),
    }];
    e.upsert(c).unwrap();
    e.install(&id).unwrap();
    thread::sleep(Duration::from_millis(300));
    e.stop(&id).unwrap();
    wait(&e, &id, "stopped");
    assert!(!e.view(&id).unwrap().config.installed);
    assert!(e.start(&id).is_err());
}

#[test]
fn bundled_webapp_examples_install_serve_change_port_and_stop() {
    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples");
    for name in ["node-webapp", "python-webapp"] {
        let (root, e) = engine();
        let project = root.path().join("project with spaces");
        fs::create_dir(&project).unwrap();
        for file in fs::read_dir(examples.join(name)).unwrap() {
            let file = file.unwrap();
            if file.file_type().unwrap().is_file() {
                fs::copy(file.path(), project.join(file.file_name())).unwrap();
            }
        }
        let mut c =
            launcher_core::manifest::read(project.join("launcher.yaml").to_str().unwrap()).unwrap();
        c.port = port();
        // Use the verified bundled uv and existing CI Python; no network/model downloads.
        if name == "python-webapp" {
            let uv = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/uv");
            for spec in c.install_steps.iter_mut().chain(c.run.iter_mut()) {
                spec.program = uv.display().to_string();
                spec.env.insert("UV_OFFLINE".into(), "true".into());
                spec.env
                    .insert("UV_PYTHON".into(), "/usr/bin/python3".into());
                spec.env
                    .insert("UV_PYTHON_DOWNLOADS".into(), "never".into());
            }
        }
        let id = e.upsert(c).unwrap();
        e.install(&id).unwrap();
        wait(&e, &id, "stopped");
        assert!(e.view(&id).unwrap().config.installed);
        for _ in 0..2 {
            e.start(&id).unwrap();
            wait(&e, &id, "running");
            let view = e.view(&id).unwrap();
            let response = reqwest::blocking::get(&view.endpoint).unwrap();
            assert_eq!(response.status(), 200);
            assert!(response.text().unwrap().contains("Web App is running"));
            e.stop(&id).unwrap();
            wait(&e, &id, "stopped");
            ensure_port_free(view.config.port).unwrap();
            let mut c = view.config;
            c.port = port();
            e.upsert(c).unwrap();
        }
        e.shutdown();
    }
}
