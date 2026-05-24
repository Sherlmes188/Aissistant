#[derive(Debug, Clone, Copy)]
pub enum ControlEvent {
    ShowWindow,
    ToggleWindow,
    QuitRequested,
}

#[cfg(windows)]
mod windows_impl {
    use super::ControlEvent;
    use eframe::egui;
    use std::ptr::null_mut;
    use std::sync::mpsc::Sender;
    use std::sync::Mutex;
    use std::sync::OnceLock;
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, HWND, LPARAM, LRESULT, POINT,
        WPARAM,
    };
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::Threading::{CreateMutexW, ReleaseMutex};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT,
    };
    use windows_sys::Win32::UI::Shell::{
        Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
        NOTIFYICONDATAW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
        DispatchMessageW, FindWindowW, GetCursorPos, GetMessageW, IsIconic, IsWindowVisible,
        LoadIconW, LoadImageW, PostMessageW, PostQuitMessage, RegisterClassW, SetForegroundWindow,
        ShowWindow, TrackPopupMenu, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, HICON,
        HMENU, IDI_APPLICATION, IMAGE_ICON, LR_LOADFROMFILE, MF_SEPARATOR, MF_STRING, MSG,
        SW_RESTORE, SW_SHOW, TPM_RIGHTBUTTON, WM_APP, WM_COMMAND, WM_DESTROY, WM_HOTKEY,
        WM_LBUTTONUP, WM_RBUTTONUP, WM_USER, WNDCLASSW, WS_OVERLAPPED,
    };

    const HOTKEY_ID: i32 = 0xA155;
    const TRAY_ID: u32 = 0xA155;
    const WM_TRAY_ICON: u32 = WM_USER + 0x155;
    const CMD_TOGGLE: usize = 1001;
    const CMD_EXIT: usize = 1002;
    const APP_TITLE: &str = "Aissistant";
    const MESSAGE_TITLE: &str = "AissistantControlWindow";
    const SINGLE_INSTANCE_MUTEX: &str = "Local\\Aissistant.SingleInstance";
    const WM_SHUTDOWN: u32 = WM_APP + 0x155;
    const WM_SHOW_EXISTING: u32 = WM_APP + 0x156;

    static SENDER: OnceLock<Sender<ControlEvent>> = OnceLock::new();
    static EGUI_CTX: OnceLock<egui::Context> = OnceLock::new();
    static MESSAGE_HWND: OnceLock<usize> = OnceLock::new();
    static TRAY_ICON_PATH: OnceLock<String> = OnceLock::new();
    static ACTIVE_HOTKEY: OnceLock<Mutex<Option<Hotkey>>> = OnceLock::new();

    #[derive(Debug, Clone, Copy)]
    struct Hotkey {
        modifiers: u32,
        key: u32,
    }

    pub struct SingleInstanceGuard {
        handle: HANDLE,
    }

    impl Drop for SingleInstanceGuard {
        fn drop(&mut self) {
            unsafe {
                ReleaseMutex(self.handle);
                CloseHandle(self.handle);
            }
        }
    }

    pub fn acquire_single_instance() -> Option<SingleInstanceGuard> {
        unsafe {
            let handle = CreateMutexW(null_mut(), 1, wide(SINGLE_INSTANCE_MUTEX).as_ptr());
            if handle.is_null() {
                return None;
            }

            if GetLastError() == ERROR_ALREADY_EXISTS {
                CloseHandle(handle);
                None
            } else {
                Some(SingleInstanceGuard { handle })
            }
        }
    }

    pub fn show_existing_instance() {
        unsafe {
            if let Some(hwnd) = find_message_window() {
                PostMessageW(hwnd, WM_SHOW_EXISTING, 0, 0);
            } else {
                show_main_window();
            }
        }
    }

    pub fn start_control_thread(
        sender: Sender<ControlEvent>,
        egui_ctx: egui::Context,
        hotkey: String,
        icon_path: Option<String>,
    ) {
        let _ = SENDER.set(sender);
        let _ = EGUI_CTX.set(egui_ctx);
        if let Some(path) = &icon_path {
            let _ = TRAY_ICON_PATH.set(path.clone());
        }
        std::thread::spawn(move || unsafe {
            let hwnd = create_message_window();
            if hwnd.is_null() {
                return;
            }
            let _ = MESSAGE_HWND.set(hwnd as usize);

            add_or_update_tray_icon(hwnd, &hotkey, icon_path.as_deref(), false);
            let active_hotkey = register_hotkey(hwnd, &hotkey).unwrap_or(Hotkey {
                modifiers: MOD_CONTROL | MOD_NOREPEAT,
                key: 0x20,
            });
            let _ = ACTIVE_HOTKEY.set(Mutex::new(Some(active_hotkey)));

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            UnregisterHotKey(hwnd, HOTKEY_ID);
            delete_tray_icon(hwnd);
        });
    }

    pub fn update_hotkey(hotkey: &str) -> Result<(), String> {
        let hwnd = MESSAGE_HWND
            .get()
            .copied()
            .map(|value| value as HWND)
            .ok_or_else(|| "hotkey window is not ready yet".to_string())?;
        let hotkey_config = parse_hotkey(hotkey)?;

        unsafe {
            let previous = ACTIVE_HOTKEY
                .get_or_init(|| Mutex::new(None))
                .lock()
                .ok()
                .and_then(|guard| *guard);

            UnregisterHotKey(hwnd, HOTKEY_ID);
            if register_hotkey_raw(hwnd, hotkey_config) == 0 {
                if let Some(previous) = previous {
                    let _ = register_hotkey_raw(hwnd, previous);
                }
                return Err(format!("failed to register hotkey: {hotkey_config:?}"));
            }

            if let Some(active) = ACTIVE_HOTKEY.get() {
                if let Ok(mut active) = active.lock() {
                    *active = Some(hotkey_config);
                }
            }
            add_or_update_tray_icon(hwnd, hotkey, TRAY_ICON_PATH.get().map(String::as_str), true);
        }
        Ok(())
    }

    pub fn shutdown_control_thread() {
        if let Some(hwnd) = MESSAGE_HWND.get().copied().map(|value| value as HWND) {
            unsafe {
                PostMessageW(hwnd, WM_SHUTDOWN, 0, 0);
            }
        }
    }

    unsafe fn create_message_window() -> HWND {
        let instance = GetModuleHandleW(null_mut());
        let class_name = wide("AissistantTrayWindow");
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: class_name.as_ptr(),
            ..Default::default()
        };
        RegisterClassW(&wc);

        CreateWindowExW(
            0,
            class_name.as_ptr(),
            wide(MESSAGE_TITLE).as_ptr(),
            WS_OVERLAPPED,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            null_mut(),
            0 as HMENU,
            instance,
            null_mut(),
        )
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_HOTKEY if wparam as i32 == HOTKEY_ID => {
                toggle_main_window();
                0
            }
            WM_TRAY_ICON if lparam as u32 == WM_LBUTTONUP => {
                toggle_main_window();
                0
            }
            WM_TRAY_ICON if lparam as u32 == WM_RBUTTONUP => {
                show_tray_menu(hwnd);
                0
            }
            WM_COMMAND if (wparam & 0xffff) == CMD_TOGGLE => {
                toggle_main_window();
                0
            }
            WM_COMMAND if (wparam & 0xffff) == CMD_EXIT => {
                request_quit();
                0
            }
            WM_SHUTDOWN => {
                DestroyWindow(hwnd);
                0
            }
            WM_SHOW_EXISTING => {
                let _ = show_main_window();
                send_event(ControlEvent::ShowWindow);
                0
            }
            WM_DESTROY => {
                delete_tray_icon(hwnd);
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    unsafe fn toggle_main_window() {
        if show_main_window() {
            send_event(ControlEvent::ShowWindow);
        } else {
            send_event(ControlEvent::ToggleWindow);
        }
    }

    unsafe fn request_quit() {
        let _ = show_main_window();
        send_event(ControlEvent::QuitRequested);
    }

    unsafe fn show_tray_menu(hwnd: HWND) {
        let menu = CreatePopupMenu();
        if menu.is_null() {
            return;
        }

        AppendMenuW(menu, MF_STRING, CMD_TOGGLE, wide("Show / Hide").as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());
        AppendMenuW(menu, MF_STRING, CMD_EXIT, wide("Exit").as_ptr());

        let mut point = POINT::default();
        GetCursorPos(&mut point);
        SetForegroundWindow(hwnd);
        TrackPopupMenu(menu, TPM_RIGHTBUTTON, point.x, point.y, 0, hwnd, null_mut());
        DestroyMenu(menu);
    }

    unsafe fn find_main_window() -> Option<HWND> {
        let hwnd = FindWindowW(null_mut(), wide(APP_TITLE).as_ptr());
        (!hwnd.is_null()).then_some(hwnd)
    }

    unsafe fn find_message_window() -> Option<HWND> {
        let hwnd = FindWindowW(null_mut(), wide(MESSAGE_TITLE).as_ptr());
        (!hwnd.is_null()).then_some(hwnd)
    }

    unsafe fn show_main_window() -> bool {
        let Some(hwnd) = find_main_window() else {
            return false;
        };

        if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_SHOW);
            ShowWindow(hwnd, SW_RESTORE);
            SetForegroundWindow(hwnd);
            true
        } else {
            SetForegroundWindow(hwnd);
            false
        }
    }

    unsafe fn register_hotkey(hwnd: HWND, hotkey: &str) -> Result<Hotkey, String> {
        let hotkey = parse_hotkey(hotkey)?;
        if register_hotkey_raw(hwnd, hotkey) == 0 {
            return Err(format!("failed to register hotkey: {hotkey:?}"));
        }
        Ok(hotkey)
    }

    unsafe fn register_hotkey_raw(hwnd: HWND, hotkey: Hotkey) -> i32 {
        RegisterHotKey(hwnd, HOTKEY_ID, hotkey.modifiers | MOD_NOREPEAT, hotkey.key)
    }

    fn parse_hotkey(value: &str) -> Result<Hotkey, String> {
        let parts = value
            .split(['+', ' '])
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();

        if parts.len() < 2 {
            return Err(
                "hotkey must include a modifier and a key, for example Ctrl+Space".to_string(),
            );
        }

        let mut modifiers = 0;
        let mut key = None;

        for part in parts {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= MOD_CONTROL,
                "alt" => modifiers |= MOD_ALT,
                "shift" => modifiers |= MOD_SHIFT,
                "space" => key = Some(0x20),
                "enter" => key = Some(0x0D),
                "tab" => key = Some(0x09),
                other if other.len() == 1 => {
                    let ch = other.chars().next().unwrap().to_ascii_uppercase();
                    if ch.is_ascii_alphanumeric() {
                        key = Some(ch as u32);
                    } else {
                        return Err(format!("unsupported hotkey key: {part}"));
                    }
                }
                _ => return Err(format!("unsupported hotkey part: {part}")),
            }
        }

        let key = key.ok_or_else(|| "hotkey key is missing".to_string())?;
        if modifiers == 0 {
            return Err("hotkey must include Ctrl, Alt, or Shift".to_string());
        }

        Ok(Hotkey { modifiers, key })
    }

    fn send_event(event: ControlEvent) {
        if let Some(sender) = SENDER.get() {
            let _ = sender.send(event);
        }
        if let Some(ctx) = EGUI_CTX.get() {
            ctx.request_repaint();
        }
    }

    unsafe fn add_or_update_tray_icon(
        hwnd: HWND,
        hotkey: &str,
        icon_path: Option<&str>,
        modify: bool,
    ) {
        let icon =
            load_custom_icon(icon_path).unwrap_or_else(|| LoadIconW(null_mut(), IDI_APPLICATION));
        let mut data = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: TRAY_ID,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
            uCallbackMessage: WM_TRAY_ICON,
            hIcon: icon,
            ..Default::default()
        };

        let tip = wide(&format!("Aissistant - {hotkey}"));
        for (target, source) in data.szTip.iter_mut().zip(tip.into_iter()) {
            *target = source;
        }

        Shell_NotifyIconW(if modify { NIM_MODIFY } else { NIM_ADD }, &mut data);
    }

    unsafe fn load_custom_icon(icon_path: Option<&str>) -> Option<HICON> {
        let path = icon_path?;
        let handle = LoadImageW(
            null_mut(),
            wide(path).as_ptr(),
            IMAGE_ICON,
            16,
            16,
            LR_LOADFROMFILE,
        );
        (!handle.is_null()).then_some(handle)
    }

    unsafe fn delete_tray_icon(hwnd: HWND) {
        let mut data = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: TRAY_ID,
            ..Default::default()
        };
        Shell_NotifyIconW(NIM_DELETE, &mut data);
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[cfg(not(windows))]
mod windows_impl {
    use super::ControlEvent;
    use std::sync::mpsc::Sender;

    pub struct SingleInstanceGuard;

    pub fn acquire_single_instance() -> Option<SingleInstanceGuard> {
        Some(SingleInstanceGuard)
    }

    pub fn show_existing_instance() {}

    pub fn start_control_thread(
        _sender: Sender<ControlEvent>,
        _egui_ctx: eframe::egui::Context,
        _hotkey: String,
        _icon_path: Option<String>,
    ) {
    }

    pub fn update_hotkey(_hotkey: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn shutdown_control_thread() {}
}

pub use windows_impl::{
    acquire_single_instance, show_existing_instance, shutdown_control_thread, start_control_thread,
    update_hotkey,
};
