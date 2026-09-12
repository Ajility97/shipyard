use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use tauri::{LogicalPosition, LogicalSize, Manager, WebviewWindow, Window, WindowEvent};

use crate::commands::AppState;
use crate::models::{WindowState, MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH};
use crate::persist;

static SAVE_GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn apply(window: &WebviewWindow, state: &WindowState) {
    let width = state.width.max(MIN_WINDOW_WIDTH);
    let height = state.height.max(MIN_WINDOW_HEIGHT);
    let _ = window.set_size(LogicalSize::new(width, height));
    if position_is_visible(window, state.x, state.y) {
        let _ = window.set_position(LogicalPosition::new(state.x, state.y));
    }
    if state.maximized {
        let _ = window.maximize();
    }
}

pub fn handle_event(window: &Window, event: &WindowEvent) {
    if window.label() != "main" {
        return;
    }
    match event {
        WindowEvent::Moved(_) | WindowEvent::Resized(_) => schedule_save(window.clone()),
        WindowEvent::CloseRequested { .. } => {
            SAVE_GENERATION.fetch_add(1, Ordering::SeqCst);
            let _ = persist_now(window);
        }
        _ => {}
    }
}

fn schedule_save(window: Window) {
    let generation = SAVE_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(350));
        if SAVE_GENERATION.load(Ordering::SeqCst) != generation {
            return;
        }
        let _ = persist_now(&window);
    });
}

fn persist_now(window: &Window) -> Result<(), String> {
    if window.is_minimized().unwrap_or(false) {
        return Ok(());
    }
    let next = capture(window)?;
    let handle = window.app_handle();
    let state = handle.state::<AppState>();
    let mut data = state.data.lock().map_err(|err| err.to_string())?;
    if data.window.as_ref() == Some(&next) {
        return Ok(());
    }
    data.window = Some(next);
    persist::save(handle, &data)
}

fn capture(window: &Window) -> Result<WindowState, String> {
    let scale = window.scale_factor().map_err(|err| err.to_string())?;
    let size = window
        .inner_size()
        .map_err(|err| err.to_string())?
        .to_logical::<u32>(scale);
    let position = window
        .outer_position()
        .map_err(|err| err.to_string())?
        .to_logical::<i32>(scale);
    Ok(WindowState {
        x: position.x,
        y: position.y,
        width: size.width.max(MIN_WINDOW_WIDTH),
        height: size.height.max(MIN_WINDOW_HEIGHT),
        maximized: window.is_maximized().unwrap_or(false),
    })
}

fn position_is_visible(window: &WebviewWindow, x: i32, y: i32) -> bool {
    let Ok(scale) = window.scale_factor() else {
        return true;
    };
    let Ok(monitors) = window.available_monitors() else {
        return true;
    };
    if monitors.is_empty() {
        return true;
    }
    let physical = LogicalPosition::new(x, y).to_physical::<i32>(scale);
    monitors.iter().any(|monitor| {
        let origin = monitor.position();
        let size = monitor.size();
        let left = origin.x;
        let top = origin.y;
        let right = origin.x.saturating_add(size.width as i32);
        let bottom = origin.y.saturating_add(size.height as i32);
        physical.x >= left - 48
            && physical.x < right - 80
            && physical.y >= top
            && physical.y < bottom - 48
    })
}
