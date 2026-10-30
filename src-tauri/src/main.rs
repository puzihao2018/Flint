#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use launcher_core::engine::{Config, Engine, RemovalOptions, RemovalPreview, View};
use launcher_core::preferences::{self, Language, PreferenceStore, Preferences};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State,
};

struct NativeMenu {
    show: MenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
}
fn update_language(app: &tauri::AppHandle, prefs: &Preferences) -> Result<(), String> {
    let menu = app.state::<NativeMenu>();
    menu.show
        .set_text(preferences::text(&prefs.locale, "打开 Flint"))
        .map_err(|e| e.to_string())?;
    menu.quit
        .set_text(preferences::text(&prefs.locale, "停止服务并退出"))
        .map_err(|e| e.to_string())?;
    if let Some(tray) = app.tray_by_id("flint") {
        tray.set_tooltip(Some(preferences::text(&prefs.locale, "Flint · 本地服务")))
            .map_err(|e| e.to_string())?;
    }
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_title(&preferences::text(&prefs.locale, "Flint · 本地服务"))
            .map_err(|e| e.to_string())?;
    }
    app.emit("preferences-changed", prefs)
        .map_err(|e| e.to_string())
}
#[tauri::command]
fn get_preferences(
    app: tauri::AppHandle,
    store: State<PreferenceStore>,
    system_locale: String,
) -> Result<Preferences, String> {
    let prefs = store.set(None, &system_locale)?;
    update_language(&app, &prefs)?;
    Ok(prefs)
}
#[tauri::command]
fn set_language(
    app: tauri::AppHandle,
    store: State<PreferenceStore>,
    language: Language,
    system_locale: String,
) -> Result<Preferences, String> {
    let prefs = store.set(Some(language), &system_locale)?;
    update_language(&app, &prefs)?;
    Ok(prefs)
}

#[tauri::command]
fn snapshot(engine: State<Engine>) -> serde_json::Value {
    serde_json::json!({"services":engine.views(),"catalog":engine.catalog(),"dataRoot":engine.root(),"platform":std::env::consts::OS})
}
#[tauri::command]
fn save_service(engine: State<Engine>, config: Config) -> Result<String, String> {
    engine.upsert(config)
}
#[tauri::command]
fn add_release(engine: State<Engine>, recipe: String, version: String) -> Result<String, String> {
    engine.add_release(&recipe, &version)
}
#[tauri::command]
fn service_action(engine: State<Engine>, id: String, action: String) -> Result<(), String> {
    match action.as_str() {
        "install" => engine.install(&id),
        "start" => engine.start(&id),
        "stop" => engine.stop(&id),
        "remove" => engine.remove(&id),
        "acknowledge" => engine.acknowledge(&id),
        _ => Err("不支持的操作".into()),
    }
}
#[tauri::command]
fn service_logs(engine: State<Engine>, id: String) -> Result<String, String> {
    engine.logs(&id)
}
#[tauri::command]
async fn removal_preview(engine: State<'_, Engine>, id: String) -> Result<RemovalPreview, String> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || engine.removal_preview(&id))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn remove_service(
    engine: State<'_, Engine>,
    id: String,
    options: RemovalOptions,
) -> Result<(), String> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || engine.remove_with_options(&id, options))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
fn open_service(engine: State<Engine>, id: String, target: String) -> Result<(), String> {
    let v: View = engine.view(&id)?;
    match target.as_str() {
        "endpoint" => {
            if v.status != "running" {
                return Err("服务尚未通过检查".into());
            }
            open::that(&v.endpoint).map_err(|e| e.to_string())
        }
        "docs" => open::that(format!("http://127.0.0.1:{}/docs", v.config.port))
            .map_err(|e| e.to_string()),
        "source" if v.config.recipe == "mineru" => {
            open::that("https://github.com/opendatalab/MinerU").map_err(|e| e.to_string())
        }
        "logs" => open::that(engine.log_path(&id)?).map_err(|e| e.to_string()),
        _ => Err("不支持的链接".into()),
    }
}
#[tauri::command]
fn show_main(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        #[cfg(target_os = "macos")]
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
    if let Some(w) = app.get_webview_window("tray") {
        let _ = w.hide();
    }
}
#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        app.state::<Engine>().shutdown();
        app.exit(0);
    });
}

#[tauri::command]
fn read_manifest(path: String) -> Result<Config, String> {
    launcher_core::manifest::read(&path)
}

#[tauri::command]
fn toggle_quick_controls(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("tray") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
            return;
        }
        if let Some(rect) = app
            .tray_by_id("flint")
            .and_then(|t| t.rect().ok().flatten())
        {
            let sf = w.scale_factor().unwrap_or(1.0);
            let p = rect.position.to_physical::<f64>(sf);
            let sz = rect.size.to_physical::<f64>(sf);
            let _ = w.set_position(tauri::PhysicalPosition::new(
                (p.x + sz.width - 390.0 * sf).max(0.0),
                p.y + sz.height + 6.0,
            ));
        }
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn main() {
    let quitting = Arc::new(AtomicBool::new(false));
    let q = quitting.clone();
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            show_main(app.clone())
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let root = std::env::var_os("FLINT_DATA_DIR")
                .map(PathBuf::from)
                .unwrap_or(app.path().app_data_dir()?);
            let uv = app.path().resource_dir()?.join("resources/uv");
            let store = PreferenceStore::new(root.clone()).map_err(std::io::Error::other)?;
            let initial = store.get();
            app.manage(store);
            let engine = Engine::new(root, uv).map_err(std::io::Error::other)?;
            app.manage(engine);
            let tray_window = tauri::WebviewWindowBuilder::new(
                app,
                "tray",
                tauri::WebviewUrl::App("index.html?tray=1".into()),
            )
            .title("Flint")
            .inner_size(390.0, 480.0)
            .decorations(false)
            .resizable(false)
            .visible(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .build()?;
            let tw = tray_window.clone();
            tray_window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    let _ = tw.hide();
                }
            });
            let show = MenuItem::with_id(
                app,
                "show",
                preferences::text(&initial.locale, "打开 Flint"),
                true,
                None::<&str>,
            )?;
            let quit = MenuItem::with_id(
                app,
                "quit",
                preferences::text(&initial.locale, "停止服务并退出"),
                true,
                None::<&str>,
            )?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            app.manage(NativeMenu { show, quit });
            TrayIconBuilder::with_id("flint")
                .icon(tauri::image::Image::from_bytes(include_bytes!(
                    "../icons/tray-flint.png"
                ))?)
                .icon_as_template(true)
                .tooltip(preferences::text(&initial.locale, "Flint · 本地服务"))
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main(app.clone()),
                    "quit" => quit_app(app.clone()),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_quick_controls(tray.app_handle().clone());
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
                // Only the main window controls Dock presence. The menu-bar
                // panel must remain usable without bringing the Dock icon back.
                #[cfg(target_os = "macos")]
                if window.label() == "main" {
                    let _ = window
                        .app_handle()
                        .set_activation_policy(tauri::ActivationPolicy::Accessory);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            snapshot,
            get_preferences,
            set_language,
            save_service,
            add_release,
            service_action,
            service_logs,
            removal_preview,
            remove_service,
            open_service,
            show_main,
            quit_app,
            read_manifest,
            toggle_quick_controls
        ])
        .build(tauri::generate_context!())
        .expect("Flint failed to initialize");
    app.run(move |app, event| {
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = event {
            show_main(app.clone());
        }

        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            if !q.swap(true, Ordering::SeqCst) {
                api.prevent_exit();
                quit_app(app.clone());
            }
        }
    });
}
