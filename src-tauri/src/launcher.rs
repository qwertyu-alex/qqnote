//! Caps Lock shows and hides qqnote, without Hyperkey or Raycast.
//!
//! macOS can't use Caps Lock as a shortcut, so while qqnote runs Caps Lock is remapped to F13
//! on every keyboard (through IOKit's UserKeyMapping, like `hidutil` does) and F13 is
//! registered as a global shortcut. Caps Lock itself is kept off, since the key can't turn it
//! off while it's remapped. qqnote keeps running when its window is closed, and can open at
//! login, so the key always works.

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
                    // In case something else turned Caps Lock on: the key can't turn it off.
                    #[cfg(target_os = "macos")]
                    caps_lock::turn_off();
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
    #[cfg(target_os = "macos")]
    warn_if_caps_lock_is_taken(app);
}

/// Says so, once per run, if something else uses Caps Lock and it might not open qqnote.
///
/// Checked when qqnote comes forward rather than at launch, since at login the other app may
/// not have started yet.
#[cfg(target_os = "macos")]
fn warn_if_caps_lock_is_taken<R: Runtime>(app: &AppHandle<R>) {
    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

    static WARNED: AtomicBool = AtomicBool::new(false);
    if WARNED.load(Ordering::SeqCst) {
        return;
    }
    let Some(reason) = caps_lock::conflict() else {
        return;
    };
    WARNED.store(true, Ordering::SeqCst);
    app.dialog()
        .message(reason)
        .title("Caps Lock might not open qqnote")
        .kind(MessageDialogKind::Warning)
        .show(|_| {});
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
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
    use core_foundation::number::CFNumber;
    use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop, CFRunLoopSource};
    use core_foundation::string::{CFString, CFStringRef};
    use objc2_app_kit::NSRunningApplication;
    use objc2_foundation::NSString;
    use std::collections::BTreeMap;
    use std::ffi::{c_char, c_void};
    use std::sync::Mutex;
    use std::time::Duration;

    // HID usages as UserKeyMapping expects them: (usage page 7, keyboard) << 32 | usage.
    const CAPS_LOCK: i64 = 0x7_0000_0039;
    const F13: i64 = 0x7_0000_0068;
    const USAGE_PAGE_GENERIC_DESKTOP: u32 = 1;
    const USAGE_KEYBOARD: u32 = 6;

    /// Hyperkey remaps Caps Lock to F13 too, then catches F13 with an event tap to turn it
    /// into its Hyper key, before qqnote's shortcut can see it.
    const HYPERKEY: &str = "com.knollsoft.Hyperkey";

    /// The keyboards qqnote remapped, by registry ID, and where their Caps Lock went before.
    static REPLACED: Mutex<BTreeMap<u64, Option<i64>>> = Mutex::new(BTreeMap::new());

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
        ) -> u8;
        fn IOHIDServiceClientGetRegistryID(service: IOHIDServiceClientRef) -> CFTypeRef;
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

        fn IORegistryEntryIDMatching(entry_id: u64) -> CFDictionaryRef;
        fn IOServiceGetMatchingService(main_port: u32, matching: CFDictionaryRef) -> u32;
        fn IORegistryEntryCreateCFProperty(
            entry: u32,
            key: CFStringRef,
            allocator: CFAllocatorRef,
            options: u32,
        ) -> CFTypeRef;
    }

    /// Remaps now, and again whenever a keyboard is connected.
    pub fn remap_while_running() {
        remap_keyboards();
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

    /// Puts back the Caps Lock mappings qqnote replaced.
    pub fn restore() {
        // Hyperkey needs Caps Lock → F13 too, even if it set it after qqnote did.
        if hyperkey_running() {
            return;
        }
        let replaced = REPLACED.lock().unwrap();
        for_each_keyboard(|service, id| {
            let Some(&original) = replaced.get(&id) else {
                return;
            };
            let mappings = user_key_mappings(service);
            if caps_lock_destination(&mappings) == Some(F13) {
                set_caps_lock_destination(service, mappings, original);
            }
        });
    }

    /// Turns Caps Lock off on every keyboard.
    pub fn turn_off() {
        for_each_keyboard(|service, _| turn_off_keyboard(service));
    }

    /// Why Caps Lock might not reach qqnote's shortcut, if something else is using it.
    pub fn conflict() -> Option<&'static str> {
        if hyperkey_running() {
            return Some(
                "Hyperkey is also using Caps Lock, and it catches the key before qqnote can. \
                 Quit Hyperkey, and stop it opening at login, to use Caps Lock with qqnote.",
            );
        }
        let mut in_system_settings = false;
        for_each_keyboard(|_, id| in_system_settings |= remapped_in_system_settings(id));
        in_system_settings.then_some(
            "Caps Lock is set to do something else in System Settings ▸ Keyboard ▸ Keyboard \
             Shortcuts ▸ Modifier Keys, so it can't open qqnote. Set it back to Caps Lock there.",
        )
    }

    extern "C" fn service_added(_refcon: *mut c_void, iterator: u32) {
        // SAFETY: `iterator` is the one IOKit just notified us about.
        unsafe { drain(iterator) };
        // The HID system picks up the new keyboard a moment after IOKit announces it.
        std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(1));
            remap_keyboards();
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

    /// Remaps Caps Lock to F13 on every keyboard, keeping its other mappings, and turns Caps
    /// Lock off, since the key can't do that anymore.
    fn remap_keyboards() {
        let hyperkey = hyperkey_running();
        let mut replaced = REPLACED.lock().unwrap();
        for_each_keyboard(|service, id| {
            let mappings = user_key_mappings(service);
            match caps_lock_destination(&mappings) {
                // Already remapped: by this qqnote, by one that didn't get to quit (so it's
                // ours to undo), or by Hyperkey (so it isn't).
                Some(F13) => {
                    if !hyperkey {
                        replaced.entry(id).or_insert(None);
                    }
                }
                original => {
                    replaced.entry(id).or_insert(original);
                    set_caps_lock_destination(service, mappings, Some(F13));
                }
            }
            turn_off_keyboard(service);
        });
    }

    fn hyperkey_running() -> bool {
        objc2::rc::autoreleasepool(|_| {
            !NSRunningApplication::runningApplicationsWithBundleIdentifier(&NSString::from_str(
                HYPERKEY,
            ))
            .is_empty()
        })
    }

    /// Calls `f` with each keyboard the HID system knows about, and its registry ID.
    ///
    /// The service refs are only valid during the call, so they mustn't escape `f`.
    fn for_each_keyboard(mut f: impl FnMut(IOHIDServiceClientRef, u64)) {
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
                let id = IOHIDServiceClientGetRegistryID(service);
                if id.is_null() {
                    continue;
                }
                let id = CFType::wrap_under_get_rule(id)
                    .downcast_into::<CFNumber>()
                    .and_then(|id| id.to_i64());
                if let Some(id) = id {
                    f(service, id as u64);
                }
            }
        }
    }

    /// The keyboard's UserKeyMapping entries.
    fn user_key_mappings(service: IOHIDServiceClientRef) -> Vec<CFType> {
        let key = CFString::from_static_string("UserKeyMapping");
        // SAFETY: `service` comes from `for_each_keyboard`; the copy is released once.
        unsafe {
            let mappings = IOHIDServiceClientCopyProperty(service, key.as_concrete_TypeRef());
            if mappings.is_null() {
                return Vec::new();
            }
            array_items(CFType::wrap_under_create_rule(mappings))
        }
    }

    /// Replaces the keyboard's Caps Lock mapping, keeping its others. `None` removes it.
    fn set_caps_lock_destination(
        service: IOHIDServiceClientRef,
        mappings: Vec<CFType>,
        destination: Option<i64>,
    ) {
        // Caps Lock maps to one key at most, so this replaces e.g. Caps Lock → Escape.
        let mut mappings: Vec<CFType> = mappings
            .into_iter()
            .filter(|mapping| mapping_ends(mapping).0 != Some(CAPS_LOCK))
            .collect();
        if let Some(destination) = destination {
            mappings.push(caps_lock_to(destination).into_CFType());
        }
        let key = CFString::from_static_string("UserKeyMapping");
        let mappings = CFArray::from_CFTypes(&mappings);
        // SAFETY: `service` comes from `for_each_keyboard`.
        let set = unsafe {
            IOHIDServiceClientSetProperty(
                service,
                key.as_concrete_TypeRef(),
                mappings.as_CFTypeRef(),
            )
        };
        if set == 0 {
            eprintln!("Couldn't update a keyboard's Caps Lock mapping");
        }
    }

    /// Turns Caps Lock off on a keyboard. The HID system tells apps, and turns off its light.
    fn turn_off_keyboard(service: IOHIDServiceClientRef) {
        let key = CFString::from_static_string("HIDCapsLockState");
        // SAFETY: `service` comes from `for_each_keyboard`.
        unsafe {
            IOHIDServiceClientSetProperty(
                service,
                key.as_concrete_TypeRef(),
                CFBoolean::false_value().as_CFTypeRef(),
            )
        };
    }

    /// If System Settings ▸ Modifier Keys changes Caps Lock on this keyboard. Those mappings
    /// apply before UserKeyMapping, so Caps Lock then never reaches qqnote's.
    fn remapped_in_system_settings(id: u64) -> bool {
        // The HID client can't read them back, but the keyboard's registry entry has them.
        // SAFETY: the matching dictionary is consumed by IOServiceGetMatchingService, the
        // entry is released, and the copied property is wrapped so it's released once.
        unsafe {
            let entry = IOServiceGetMatchingService(0, IORegistryEntryIDMatching(id));
            if entry == 0 {
                return false;
            }
            let key = CFString::from_static_string("HIDEventServiceProperties");
            let properties = IORegistryEntryCreateCFProperty(
                entry,
                key.as_concrete_TypeRef(),
                kCFAllocatorDefault,
                0,
            );
            IOObjectRelease(entry);
            if properties.is_null() {
                return false;
            }
            let Some(properties) =
                CFType::wrap_under_create_rule(properties).downcast_into::<CFDictionary>()
            else {
                return false;
            };
            let key = CFString::from_static_string("HIDKeyboardModifierMappingPairs");
            let Some(pairs) = properties.find(key.as_CFTypeRef()) else {
                return false;
            };
            let pairs = array_items(CFType::wrap_under_get_rule(*pairs));
            caps_lock_destination(&pairs).is_some_and(|destination| destination != CAPS_LOCK)
        }
    }

    /// The items of a CF array, or none if `value` isn't one.
    fn array_items(value: CFType) -> Vec<CFType> {
        value
            .downcast_into::<CFArray>()
            .map(|array| {
                array
                    .iter()
                    // SAFETY: array items are CF objects; this retains each one.
                    .map(|item| unsafe { CFType::wrap_under_get_rule(*item) })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Where a keyboard's mappings send Caps Lock, if they map it.
    fn caps_lock_destination(mappings: &[CFType]) -> Option<i64> {
        mappings
            .iter()
            .map(mapping_ends)
            .find(|(source, _)| *source == Some(CAPS_LOCK))
            .and_then(|(_, destination)| destination)
    }

    fn caps_lock_to(destination: i64) -> CFDictionary<CFString, CFNumber> {
        CFDictionary::from_CFType_pairs(&[
            (
                CFString::from_static_string("HIDKeyboardModifierMappingSrc"),
                CFNumber::from(CAPS_LOCK),
            ),
            (
                CFString::from_static_string("HIDKeyboardModifierMappingDst"),
                CFNumber::from(destination),
            ),
        ])
    }

    /// The (source, destination) keys of a key mapping entry.
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
