use std::{sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread, time::Duration};
use tauri::{Emitter, Manager, WebviewWindow};
use windows::Win32::{Foundation::POINT, UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO}};

const IDLE_MASK: &[u8] = include_bytes!("../../public/cats/idle.png");
const IDLE_AFTER_MS: u32 = 180_000;

#[derive(Default)]
struct AppState {
    resting: Arc<AtomicBool>,
    interactive: Arc<AtomicBool>,
    mask: Arc<Mutex<Option<(u32, u32, Vec<u8>)>>>,
}

fn load_mask() -> Option<(u32, u32, Vec<u8>)> {
    let idle = image::load_from_memory(IDLE_MASK).ok()?.to_rgba8();
    let sleep = image::load_from_memory(include_bytes!("../../public/cats/sleep.png")).ok()?.to_rgba8();
    let walk1 = image::load_from_memory(include_bytes!("../../public/cats/walk-1.png")).ok()?.to_rgba8();
    let walk2 = image::load_from_memory(include_bytes!("../../public/cats/walk-2.png")).ok()?.to_rgba8();
    let walk3 = image::load_from_memory(include_bytes!("../../public/cats/walk-3.png")).ok()?.to_rgba8();
    let (width, height) = idle.dimensions();
    let planes = [idle.as_raw(), sleep.as_raw(), walk1.as_raw(), walk2.as_raw(), walk3.as_raw()];
    let pixels = (0..width * height).map(|i| {
        planes.iter().map(|rgba| rgba[(i * 4 + 3) as usize]).max().unwrap_or(0)
    }).collect();
    Some((width, height, pixels))
}

#[tauri::command]
fn start_dragging(window: WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_resting(resting: bool, state: tauri::State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    state.resting.store(resting, Ordering::Relaxed);
    app.emit("resting-changed", resting).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_interactive(interactive: bool, state: tauri::State<'_, AppState>) {
    state.interactive.store(interactive, Ordering::Relaxed);
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) { app.exit(0); }

#[tauri::command]
fn system_idle() -> bool {
    let mut input = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    unsafe { GetLastInputInfo(&mut input).as_bool() && input.elapsed_since() >= IDLE_AFTER_MS }
}

fn in_pet_shape(window: &WebviewWindow, mask: &(u32, u32, Vec<u8>), point: POINT) -> bool {
    let Ok(position) = window.outer_position() else { return false; };
    let Ok(size) = window.outer_size() else { return false; };
    if size.width == 0 || size.height == 0 { return false; }
    let x = point.x - position.x;
    let y = point.y - position.y;
    if x < 0 || y < 0 || x as u32 >= size.width || y as u32 >= size.height { return false; }
    let source_x = (x as u32 * mask.0 / size.width).min(mask.0 - 1);
    let source_y = (y as u32 * mask.1 / size.height).min(mask.1 - 1);
    mask.2[(source_y * mask.0 + source_x) as usize] > 16
}

fn install_tray(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::TrayIconBuilder;

    let toggle = MenuItem::with_id(app, "toggle-rest", "切换待机 / 休息", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出咪咪桌宠", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle, &quit])?;
    let icon = app.default_window_icon().cloned().expect("bundle icon is configured");
    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("咪咪桌宠")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "toggle-rest" => {
                let state = app.state::<AppState>();
                let resting = !state.resting.load(Ordering::Relaxed);
                state.resting.store(resting, Ordering::Relaxed);
                let _ = app.emit("resting-changed", resting);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(move |tray, event| {
            if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                let app = tray.app_handle();
                let state = app.state::<AppState>();
                let resting = !state.resting.load(Ordering::Relaxed);
                state.resting.store(resting, Ordering::Relaxed);
                let _ = app.emit("resting-changed", resting);
            }
        })
        .build(app)?;
    Ok(())
}

fn start_desktop_monitor(app: tauri::AppHandle, window: WebviewWindow, state: Arc<AppState>) {
    thread::spawn(move || {
        let mut last_idle = false;
        loop {
            thread::sleep(Duration::from_millis(75));
            let mut input = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
            let idle = unsafe { GetLastInputInfo(&mut input).as_bool() && input.elapsed_since() >= IDLE_AFTER_MS };
            if idle != last_idle {
                last_idle = idle;
                let _ = app.emit("system-idle", idle);
            }
            let mut position = POINT { x: 0, y: 0 };
            if unsafe { windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut position) }.is_err() { continue; }
            let in_shape = state.mask.lock().ok().and_then(|mask| mask.as_ref().map(|m| in_pet_shape(&window, m, position))).unwrap_or(false);
            let interactive = state.interactive.load(Ordering::Relaxed);
            let _ = window.set_ignore_cursor_events(!interactive && !in_shape);
        }
    });
}

trait ElapsedInput {
    fn elapsed_since(&self) -> u32;
}

impl ElapsedInput for LASTINPUTINFO {
    fn elapsed_since(&self) -> u32 {
        unsafe { windows::Win32::System::SystemInformation::GetTickCount() }.wrapping_sub(self.dwTime)
    }
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            let state = app.state::<AppState>();
            *state.mask.lock().expect("hit-test mask mutex poisoned") = load_mask();
            install_tray(app)?;
            let window = app.get_webview_window("main").expect("main window is configured");
            let app_handle = app.handle().clone();
            let state = Arc::new(AppState {
                resting: state.resting.clone(),
                interactive: state.interactive.clone(),
                mask: state.mask.clone(),
            });
            start_desktop_monitor(app_handle, window, state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![start_dragging, set_resting, set_interactive, system_idle, quit_app])
        .run(tauri::generate_context!())
        .expect("failed to run Mimi Deskpet");
}
