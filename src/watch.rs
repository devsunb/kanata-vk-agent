use block2::StackBlock;
use core_foundation::{
    base::TCFType,
    data::CFDataRef,
    runloop::CFRunLoopRun,
    string::{CFString, CFStringRef},
};
use objc2::rc::Retained;
use objc2_app_kit::{
    NSRunningApplication, NSWorkspace, NSWorkspaceApplicationKey,
    NSWorkspaceDidActivateApplicationNotification,
};
use objc2_foundation::{NSDistributedNotificationCenter, NSNotification};
use std::{ffi::c_void, ptr::NonNull};

#[repr(transparent)]
struct TISInputSource(c_void);
type TISInputSourceRef = *mut TISInputSource;
#[link(name = "Carbon", kind = "framework")]
extern "C" {
    fn TISCopyCurrentKeyboardInputSource() -> TISInputSourceRef;
    fn TISGetInputSourceProperty(source: TISInputSourceRef, propertyKey: CFStringRef) -> CFDataRef;
    static kTISPropertyInputSourceID: CFStringRef;
    static kTISNotifySelectedKeyboardInputSourceChanged: CFStringRef;
}

pub fn input_source() -> String {
    unsafe {
        let src = TISCopyCurrentKeyboardInputSource();
        let src_id = TISGetInputSourceProperty(src, kTISPropertyInputSourceID) as CFStringRef;
        CFString::wrap_under_get_rule(src_id).to_string()
    }
}

pub fn frontmost_app_bundle_id() -> Option<String> {
    unsafe {
        Some(
            NSWorkspace::sharedWorkspace()
                .frontmostApplication()?
                .bundleIdentifier()?
                .to_string(),
        )
    }
}

pub fn watch(
    app_fn: impl Fn(String) + Clone + 'static,
    input_source_fn: impl Fn(String) + Clone + 'static,
) {
    unsafe {
        let app_center = NSWorkspace::sharedWorkspace().notificationCenter();
        let app_observer = app_center.addObserverForName_object_queue_usingBlock(
            Some(NSWorkspaceDidActivateApplicationNotification),
            None,
            None,
            &StackBlock::new(move |n: NonNull<NSNotification>| {
                app_fn(
                    Retained::cast::<NSRunningApplication>(
                        n.as_ref()
                            .userInfo()
                            .unwrap()
                            .objectForKey(NSWorkspaceApplicationKey)
                            .unwrap(),
                    )
                    .bundleIdentifier()
                    .unwrap()
                    .to_string(),
                )
            }),
        );

        let input_center = NSDistributedNotificationCenter::defaultCenter();
        let input_observer = input_center.addObserverForName_object_queue_usingBlock(
            Some(&*kTISNotifySelectedKeyboardInputSourceChanged.cast()),
            None,
            None,
            &StackBlock::new(move |_| input_source_fn(input_source())),
        );

        // CFRunLoopRun runs indefinitely
        CFRunLoopRun();
        app_center.removeObserver(&app_observer);
        input_center.removeObserver(&input_observer);
    }
}
