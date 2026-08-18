//! 系统托盘（AGENTS.md §7.2）。
//!
//! 托盘菜单：运行状态、今日节省、暂停/恢复、打开仪表盘、紧急还原、退出。

use std::sync::atomic::Ordering;
use std::sync::Arc;

use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{Emitter, Manager};

use crate::ProxyState;

/// 创建系统托盘。
pub fn create_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let toggle_item = MenuItemBuilder::with_id("toggle", "暂停")
        .accelerator("CmdOrCtrl+T")
        .build(app)?;
    let dashboard_item = MenuItemBuilder::with_id("dashboard", "打开仪表盘")
        .build(app)?;
    let restore_item = MenuItemBuilder::with_id("restore", "紧急还原")
        .build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "退出")
        .accelerator("CmdOrCtrl+Q")
        .build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&MenuItemBuilder::with_id("status", "TokenHusk 运行中").enabled(false).build(app)?)
        .separator()
        .item(&toggle_item)
        .item(&dashboard_item)
        .separator()
        .item(&restore_item)
        .separator()
        .item(&quit_item)
        .build()?;

    TrayIconBuilder::new()
        .icon(tauri::image::Image::from_bytes(include_bytes!("../icons/icon.svg"))?)
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "toggle" => {
                let state: State<'_, ProxyState> = app.state();
                let paused = state.paused.load(Ordering::Relaxed);
                if paused {
                    state.paused.store(false, Ordering::Relaxed);
                    toggle_item.set_text(app, "暂停").ok();
                    app.emit("proxy-status", "running").ok();
                } else {
                    state.paused.store(true, Ordering::Relaxed);
                    toggle_item.set_text(app, "恢复").ok();
                    app.emit("proxy-status", "paused").ok();
                }
            }
            "dashboard" => {
                if let Some(window) = app.get_webview_window("main") {
                    window.show().ok();
                    window.set_focus().ok();
                }
            }
            "restore" => {
                // 触发前端紧急还原对话框
                app.emit("request-restore", true).ok();
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    window.show().ok();
                    window.set_focus().ok();
                }
            }
        })
        .build(app)?;

    Ok(())
}
