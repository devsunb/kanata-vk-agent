use block2::StackBlock;
use core_foundation::runloop::CFRunLoopRun;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSRunningApplication, NSWorkspace, NSWorkspaceApplicationKey,
    NSWorkspaceDidActivateApplicationNotification,
};
use objc2_foundation::{NSNotification, NSString};
use std::{
    ptr::NonNull,
    sync::mpsc::{Receiver, Sender},
    thread,
};

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

pub fn frontmost_app_bundle_id() -> Option<Retained<NSString>> {
    unsafe {
        NSWorkspace::sharedWorkspace()
            .frontmostApplication()?
            .bundleIdentifier()
    }
}

pub fn run<F, T>(tx: Sender<Retained<NSString>>, f: F)
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
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
                tx.send(bundle_id).unwrap();
            }),
        )
    };
    let handle = thread::spawn(f);
    // CFRunLoopRun runs indefinitely
    unsafe { CFRunLoopRun() };
    handle.join().unwrap();
    unsafe { center.removeObserver(&observer) };
}

pub fn id_mode(tx: Sender<Retained<NSString>>, rx: Receiver<Retained<NSString>>) {
    run(tx, move || {
        let mut current = frontmost_app_bundle_id().unwrap().to_string();
        println!("{current}");
        for new in rx {
            let new = new.to_string();
            if current == new {
                continue;
            }
            println!("{new}");
            current = new;
        }
    });
}
