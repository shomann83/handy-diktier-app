//! Captures the digit keys 0-9 system-wide while a style-bar session is open.
//!
//! On Windows a low-level keyboard hook (`WH_KEYBOARD_LL`) runs on its own
//! thread for the duration of a session. Every digit (top row and numpad,
//! with or without held modifiers) is swallowed so it never reaches the
//! focused app; key-downs are reported through the callback. Injected events
//! pass through untouched so Handy's own paste is never blocked.
//!
//! The callback runs inside the hook procedure, which Windows times out after
//! a few hundred milliseconds — it must return quickly (hand work off to
//! another thread instead of blocking).

use std::sync::Arc;

type DigitCallback = Arc<dyn Fn(u8) + Send + Sync + 'static>;

/// A swallowed digit pressed while Alt or Win is held would leave the app
/// seeing a lone modifier tap on release, which opens the menu bar / Office
/// key-tips (Alt) or the Start menu (Win). Injecting a neutral key prevents it.
pub fn needs_menu_mask(alt_down: bool, win_down: bool) -> bool {
    alt_down || win_down
}

/// Map a Windows virtual-key code to a digit: `0x30..=0x39` (top row) and
/// `0x60..=0x69` (numpad).
pub fn digit_from_vk(vk: u32) -> Option<u8> {
    match vk {
        0x30..=0x39 => Some((vk - 0x30) as u8),
        0x60..=0x69 => Some((vk - 0x60) as u8),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use super::{digit_from_vk, needs_menu_mask, DigitCallback};
    use log::{debug, error, warn};
    use std::sync::{mpsc, Mutex};
    use std::thread::JoinHandle;
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
        KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_LWIN, VK_MENU, VK_RWIN,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, PeekMessageW, PostThreadMessageW, SetWindowsHookExW,
        UnhookWindowsHookEx, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG, PM_NOREMOVE, WH_KEYBOARD_LL,
        WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN,
    };

    /// Unassigned virtual key (same as handy-keys' menu mask): a neutral
    /// keystroke the shell and Office treat as "a key was pressed with Alt/Win".
    const MENU_MASK_VK: u16 = 0xE8;

    static CALLBACK: Mutex<Option<DigitCallback>> = Mutex::new(None);

    fn is_down(vk: VIRTUAL_KEY) -> bool {
        unsafe { GetAsyncKeyState(vk.0 as i32) as u16 & 0x8000 != 0 }
    }

    fn send_menu_mask() {
        let key = |flags: KEYBD_EVENT_FLAGS| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(MENU_MASK_VK),
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        let inputs = [key(KEYBD_EVENT_FLAGS(0)), key(KEYEVENTF_KEYUP)];
        let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
        if sent != inputs.len() as u32 {
            warn!("Digit hook: menu mask injection failed");
        }
    }
    static HOOK_THREAD: Mutex<Option<(u32, JoinHandle<()>)>> = Mutex::new(None);

    unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 {
            let event = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
            let injected = event.flags.0 & LLKHF_INJECTED.0 != 0;
            if !injected {
                if let Some(digit) = digit_from_vk(event.vkCode) {
                    let msg = wparam.0 as u32;
                    if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                        if needs_menu_mask(is_down(VK_MENU), is_down(VK_LWIN) || is_down(VK_RWIN)) {
                            send_menu_mask();
                        }
                        let callback = CALLBACK.lock().ok().and_then(|c| c.clone());
                        if let Some(callback) = callback {
                            callback(digit);
                        }
                    }
                    // Swallow both down and up so the digit never reaches the app.
                    return LRESULT(1);
                }
            }
        }
        CallNextHookEx(None, code, wparam, lparam)
    }

    pub fn start(on_digit: DigitCallback) {
        if let Ok(mut cb) = CALLBACK.lock() {
            *cb = Some(on_digit);
        }

        let mut thread = HOOK_THREAD.lock().unwrap_or_else(|e| e.into_inner());
        if thread.is_some() {
            return;
        }

        let (tx, rx) = mpsc::channel::<u32>();
        let handle = std::thread::spawn(move || unsafe {
            // Force the message queue into existence before reporting our id,
            // so a stop() right after start() can always post WM_QUIT.
            let mut msg = MSG::default();
            let _ = PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE);
            let _ = tx.send(GetCurrentThreadId());

            let hook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0) {
                Ok(hook) => hook,
                Err(e) => {
                    error!("Failed to install digit hook: {e}");
                    return;
                }
            };
            debug!("Digit hook installed");
            while GetMessageW(&mut msg, None, 0, 0).0 > 0 {}
            let _ = UnhookWindowsHookEx(hook);
            debug!("Digit hook removed");
        });

        match rx.recv() {
            Ok(thread_id) => *thread = Some((thread_id, handle)),
            Err(_) => error!("Digit hook thread exited before starting"),
        }
    }

    pub fn stop() {
        let taken = HOOK_THREAD.lock().unwrap_or_else(|e| e.into_inner()).take();
        if let Some((thread_id, handle)) = taken {
            unsafe {
                if let Err(e) = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) {
                    error!("Failed to stop digit hook thread: {e}");
                }
            }
            let _ = handle.join();
        }
        if let Ok(mut cb) = CALLBACK.lock() {
            *cb = None;
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use super::DigitCallback;

    pub fn start(_on_digit: DigitCallback) {}
    pub fn stop() {}
}

/// Start swallowing digits and report key-downs to `on_digit`. Idempotent:
/// a second call only replaces the callback.
pub fn start(on_digit: Box<dyn Fn(u8) + Send + Sync + 'static>) {
    imp::start(Arc::from(on_digit));
}

/// Stop swallowing digits. Idempotent.
pub fn stop() {
    imp::stop();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_row_digits_map() {
        for d in 0..=9u8 {
            assert_eq!(digit_from_vk(0x30 + d as u32), Some(d));
        }
    }

    #[test]
    fn numpad_digits_map() {
        for d in 0..=9u8 {
            assert_eq!(digit_from_vk(0x60 + d as u32), Some(d));
        }
    }

    #[test]
    fn menu_mask_only_when_alt_or_win_is_held() {
        assert!(!needs_menu_mask(false, false));
        assert!(needs_menu_mask(true, false));
        assert!(needs_menu_mask(false, true));
        assert!(needs_menu_mask(true, true));
    }

    #[test]
    fn other_keys_do_not_map() {
        for vk in [0x20u32, 0x41, 0x12, 0x1B, 0x2F, 0x3A, 0x5F, 0x6A] {
            assert_eq!(digit_from_vk(vk), None);
        }
    }
}
