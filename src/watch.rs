use block2::StackBlock;
use core_foundation::runloop::CFRunLoopRun;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSRunningApplication, NSWorkspace, NSWorkspaceApplicationKey,
    NSWorkspaceDidActivateApplicationNotification,
};
use objc2_foundation::{NSNotification, NSString};
use std::{ptr::NonNull, sync::mpsc::Sender};

trait Notification {
    fn bundle_id(&self) -> Option<Retained<NSString>>;
}

impl Notification for NSNotification {
    fn bundle_id(&self) -> Option<Retained<NSString>> {
        unsafe {
            Retained::cast::<NSRunningApplication>(
                self.userInfo()?.objectForKey(NSWorkspaceApplicationKey)?,
            )
            .bundleIdentifier()
        }
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

pub fn watch(tx: Sender<String>) {
    let center = unsafe { NSWorkspace::sharedWorkspace().notificationCenter() };
    let observer = unsafe {
        center.addObserverForName_object_queue_usingBlock(
            Some(NSWorkspaceDidActivateApplicationNotification),
            None,
            None,
            &StackBlock::new(move |notif: NonNull<NSNotification>| {
                let Some(bundle_id) = notif.as_ref().bundle_id() else {
                    return;
                };
                tx.send(bundle_id.to_string()).unwrap();
            }),
        )
    };
    // CFRunLoopRun runs indefinitely
    unsafe { CFRunLoopRun() };
    unsafe { center.removeObserver(&observer) };
}
