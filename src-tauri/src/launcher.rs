//! Caps Lock shows and hides qqnote, without Hyperkey or Raycast.
//!
//! macOS can't use Caps Lock as a shortcut, so while qqnote runs Caps Lock is remapped to F13
//! on every keyboard (the way System Settings ▸ Keyboard ▸ Modifier Keys remaps keys) and F13
//! is registered as a global shortcut. qqnote keeps running when its window is closed, and can
//! open at login, so the key always works.

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::menu::CheckMenuItem;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

/// Passed by the login item so qqnote starts without showing its window.
pub const BACKGROUND_ARG: &str = "--background";

pub fn setup<R: Runtime>(app: &AppHandle<R>) {
    #[cfg(target_os = "macos")]
    caps_lock::remap_while_running();

    // Holding the key repeats the press; only the first one toggles.
    let held = AtomicBool::new(false);
    let f13 = Shortcut::new(None, Code::F13);
    let result = app
        .global_shortcut()
        .on_shortcut(f13, move |app, _, event| match event.state {
            ShortcutState::Pressed => {
                if !held.swap(true, Ordering::SeqCst) {
                    toggle(app);
                }
            }
            ShortcutState::Released => held.store(false, Ordering::SeqCst),
        });
    if let Err(e) = result {
        eprintln!("Couldn't register F13 as a global shortcut: {}", e);
    }

    sync_open_at_login_item(app);

    if !std::env::args().any(|arg| arg == BACKGROUND_ARG) {
        show(app);
    }
}

/// Gives Caps Lock back. Called when qqnote exits.
pub fn teardown() {
    #[cfg(target_os = "macos")]
    caps_lock::restore();
}

/// Brings the window forward, even if it was closed or the app was hidden.
pub fn show<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    #[cfg(target_os = "macos")]
    let _ = app.show();
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

/// Hides qqnote if it's in front, otherwise brings it forward.
fn toggle<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_focused().unwrap_or(false) {
        // Hiding the app, not just the window, gives focus back to the previous app.
        #[cfg(target_os = "macos")]
        let _ = app.hide();
        #[cfg(not(target_os = "macos"))]
        let _ = window.hide();
    } else {
        show(app);
    }
}

pub fn toggle_open_at_login<R: Runtime>(app: &AppHandle<R>) {
    let autolaunch = app.autolaunch();
    let result = if autolaunch.is_enabled().unwrap_or(false) {
        autolaunch.disable()
    } else {
        autolaunch.enable()
    };
    if let Err(e) = result {
        eprintln!("Couldn't change Open at Login: {}", e);
    }
    // Clicking already flipped the checkmark; show what actually happened.
    sync_open_at_login_item(app);
}

fn sync_open_at_login_item<R: Runtime>(app: &AppHandle<R>) {
    let item: Option<CheckMenuItem<R>> = app
        .menu()
        .and_then(|menu| menu.get("file"))
        .and_then(|file| file.as_submenu()?.get("open_at_login"))
        .and_then(|item| item.as_check_menuitem().cloned());
    if let Some(item) = item {
        let _ = item.set_checked(app.autolaunch().is_enabled().unwrap_or(false));
    }
}

#[cfg(target_os = "macos")]
mod caps_lock {
    use core_foundation::array::{CFArray, CFArrayRef};
    use core_foundation::base::{kCFAllocatorDefault, CFAllocatorRef, CFType, CFTypeRef, TCFType};
    use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
    use core_foundation::number::CFNumber;
    use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop, CFRunLoopSource};
    use core_foundation::string::{CFString, CFStringRef};
    use std::ffi::{c_char, c_void};
    use std::time::Duration;

    // HID usages as UserKeyMapping expects them: (usage page 7, keyboard) << 32 | usage.
    const CAPS_LOCK: i64 = 0x7_0000_0039;
    const F13: i64 = 0x7_0000_0068;
    const USAGE_PAGE_GENERIC_DESKTOP: u32 = 1;
    const USAGE_KEYBOARD: u32 = 6;

    type IOHIDEventSystemClientRef = CFTypeRef;
    type IOHIDServiceClientRef = *const c_void;
    type IONotificationPortRef = *mut c_void;
    type IOServiceMatchingCallback = extern "C" fn(refcon: *mut c_void, iterator: u32);

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOHIDEventSystemClientCreateSimpleClient(
            allocator: CFAllocatorRef,
        ) -> IOHIDEventSystemClientRef;
        fn IOHIDEventSystemClientCopyServices(client: IOHIDEventSystemClientRef) -> CFArrayRef;
        fn IOHIDServiceClientConformsTo(
            service: IOHIDServiceClientRef,
            usage_page: u32,
            usage: u32,
        ) -> i32;
        fn IOHIDServiceClientCopyProperty(
            service: IOHIDServiceClientRef,
            key: CFStringRef,
        ) -> CFTypeRef;
        fn IOHIDServiceClientSetProperty(
            service: IOHIDServiceClientRef,
            key: CFStringRef,
            property: CFTypeRef,
        ) -> u8;

        fn IONotificationPortCreate(main_port: u32) -> IONotificationPortRef;
        fn IONotificationPortGetRunLoopSource(
            port: IONotificationPortRef,
        ) -> core_foundation::runloop::CFRunLoopSourceRef;
        fn IOServiceMatching(name: *const c_char) -> CFDictionaryRef;
        fn IOServiceAddMatchingNotification(
            port: IONotificationPortRef,
            notification_type: *const c_char,
            matching: CFDictionaryRef,
            callback: IOServiceMatchingCallback,
            refcon: *mut c_void,
            iterator: *mut u32,
        ) -> i32;
        fn IOIteratorNext(iterator: u32) -> u32;
        fn IOObjectRelease(object: u32) -> i32;
    }

    /// Remaps now, and again whenever a keyboard is connected.
    pub fn remap_while_running() {
        update_keyboards(true);
        // SAFETY: called on the main thread; the port and iterator live as long as the app.
        unsafe {
            let port = IONotificationPortCreate(0);
            let source = IONotificationPortGetRunLoopSource(port);
            CFRunLoop::get_main().add_source(
                &CFRunLoopSource::wrap_under_get_rule(source),
                kCFRunLoopCommonModes,
            );
            let mut iterator = 0;
            let result = IOServiceAddMatchingNotification(
                port,
                c"IOServiceFirstMatch".as_ptr(),
                IOServiceMatching(c"IOHIDEventService".as_ptr()),
                service_added,
                std::ptr::null_mut(),
                &mut iterator,
            );
            if result != 0 {
                eprintln!("Couldn't watch for new keyboards: {:#x}", result);
                return;
            }
            // The notification only fires once the existing services are drained.
            drain(iterator);
        }
    }

    pub fn restore() {
        update_keyboards(false);
    }

    extern "C" fn service_added(_refcon: *mut c_void, iterator: u32) {
        // SAFETY: `iterator` is the one IOKit just notified us about.
        unsafe { drain(iterator) };
        // The HID system picks up the new keyboard a moment after IOKit announces it.
        std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(1));
            update_keyboards(true);
        });
    }

    unsafe fn drain(iterator: u32) {
        loop {
            let service = IOIteratorNext(iterator);
            if service == 0 {
                break;
            }
            IOObjectRelease(service);
        }
    }

    /// Adds (or removes) the Caps Lock → F13 mapping on every keyboard, keeping its other
    /// mappings, e.g. ones from System Settings.
    fn update_keyboards(remap: bool) {
        let key = CFString::from_static_string("UserKeyMapping");
        // SAFETY: the IOKit calls get valid client/service refs, and every returned object
        // is wrapped so it's released exactly once.
        unsafe {
            let client = IOHIDEventSystemClientCreateSimpleClient(kCFAllocatorDefault);
            if client.is_null() {
                eprintln!("Couldn't connect to the HID system to remap Caps Lock");
                return;
            }
            let client = CFType::wrap_under_create_rule(client);
            let services = IOHIDEventSystemClientCopyServices(client.as_CFTypeRef());
            if services.is_null() {
                return;
            }
            let services: CFArray<*const c_void> = CFArray::wrap_under_create_rule(services);

            for service in services.iter() {
                let service = *service;
                if IOHIDServiceClientConformsTo(service, USAGE_PAGE_GENERIC_DESKTOP, USAGE_KEYBOARD)
                    == 0
                {
                    continue;
                }

                let current = IOHIDServiceClientCopyProperty(service, key.as_concrete_TypeRef());
                let current: Vec<CFType> = if current.is_null() {
                    Vec::new()
                } else {
                    CFType::wrap_under_create_rule(current)
                        .downcast_into::<CFArray>()
                        .map(|mappings| {
                            mappings
                                .iter()
                                .map(|mapping| CFType::wrap_under_get_rule(*mapping))
                                .collect()
                        })
                        .unwrap_or_default()
                };

                let remapped = current
                    .iter()
                    .any(|mapping| mapping_ends(mapping) == (Some(CAPS_LOCK), Some(F13)));
                if remapped == remap {
                    continue;
                }

                // Caps Lock maps to one key at most, so this replaces e.g. Caps Lock → Escape.
                let mut mappings: Vec<CFType> = current
                    .into_iter()
                    .filter(|mapping| mapping_ends(mapping).0 != Some(CAPS_LOCK))
                    .collect();
                if remap {
                    mappings.push(caps_lock_to_f13().into_CFType());
                }
                let mappings = CFArray::from_CFTypes(&mappings);
                if IOHIDServiceClientSetProperty(
                    service,
                    key.as_concrete_TypeRef(),
                    mappings.as_CFTypeRef(),
                ) == 0
                {
                    eprintln!("Couldn't update a keyboard's Caps Lock mapping");
                }
            }
        }
    }

    fn caps_lock_to_f13() -> CFDictionary<CFString, CFNumber> {
        CFDictionary::from_CFType_pairs(&[
            (
                CFString::from_static_string("HIDKeyboardModifierMappingSrc"),
                CFNumber::from(CAPS_LOCK),
            ),
            (
                CFString::from_static_string("HIDKeyboardModifierMappingDst"),
                CFNumber::from(F13),
            ),
        ])
    }

    /// The (source, destination) keys of a UserKeyMapping entry.
    fn mapping_ends(mapping: &CFType) -> (Option<i64>, Option<i64>) {
        let Some(mapping) = mapping.downcast::<CFDictionary>() else {
            return (None, None);
        };
        let usage = |name: &'static str| {
            let name = CFString::from_static_string(name);
            let value = mapping.find(name.as_CFTypeRef())?;
            // SAFETY: dictionary values are CF objects; this retains it while we read it.
            unsafe { CFType::wrap_under_get_rule(*value) }
                .downcast::<CFNumber>()?
                .to_i64()
        };
        (
            usage("HIDKeyboardModifierMappingSrc"),
            usage("HIDKeyboardModifierMappingDst"),
        )
    }
}
