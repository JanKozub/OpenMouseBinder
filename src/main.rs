use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoopGetCurrent, CFRunLoopRun};
use core_foundation_sys::base::CFAllocatorRef;
use core_foundation_sys::mach_port::CFMachPortRef;
use core_foundation_sys::runloop::{CFRunLoopAddSource, CFRunLoopSourceRef};
use core_graphics::event::{
    CGEvent, CGEventField, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventTapProxy, CGEventType,
};
use core_graphics::sys::CGEventRef;
use std::os::raw::c_void;
use std::ptr;
use std::ptr::null_mut;

const MOUSE_EVENT_NUMBER: CGEventField = unsafe { std::mem::transmute(3u32) };

// Callback type for the event tap.
pub type CGEventTapCallBack = extern "C" fn(
    proxy: CGEventTapProxy,
    type_: CGEventType,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    // Creates an event tap.
    fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        events_of_interest: u64,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> CFMachPortRef;

    // Enables or disables an event tap.
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);

    // Creates a run loop source from a Mach port.
    fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef,
        port: CFMachPortRef,
        order: isize,
    ) -> CFRunLoopSourceRef;

    // Create a copy of a CGEvent.
    fn CGEventCreateCopy(event: CGEventRef) -> CGEventRef;
}

// Our event tap callback function.
// It handles mouse movement, left mouse button events, and "other" mouse button events.
extern "C" fn event_callback(
    _proxy: CGEventTapProxy,
    type_: CGEventType,
    event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    unsafe {
        let copy = CGEventCreateCopy(event);
        let cg: CGEvent = std::mem::transmute(copy);

        match type_ {
            CGEventType::OtherMouseDown => {
                let button = cg.get_integer_value_field(MOUSE_EVENT_NUMBER);
                println!("Other mouse button down: button number {}", button);
                null_mut()
            }
            CGEventType::OtherMouseUp => null_mut(),
            _ => event,
        }
    }
}

fn main() {
    // Build the event mask for the events we care about.
    let mask: u64 = (1 << (CGEventType::MouseMoved as u64))
        | (1 << (CGEventType::LeftMouseDown as u64))
        | (1 << (CGEventType::LeftMouseUp as u64))
        | (1 << (CGEventType::OtherMouseDown as u64))
        | (1 << (CGEventType::OtherMouseUp as u64));

    unsafe {
        // Create an event tap at the session level.
        let event_tap: CFMachPortRef = CGEventTapCreate(
            CGEventTapLocation::Session,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            mask,
            event_callback,
            null_mut(),
        );

        if event_tap.is_null() {
            eprintln!("Failed to create event tap.");
            std::process::exit(1);
        }

        // Enable the event tap.
        CGEventTapEnable(event_tap, true);

        // Create a run loop source from the event tap.
        let run_loop_source = CFMachPortCreateRunLoopSource(ptr::null(), event_tap, 0);
        if run_loop_source.is_null() {
            eprintln!("Failed to create run loop source");
            std::process::exit(1);
        }

        // Add the run loop source to the current run loop.
        let run_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(run_loop, run_loop_source, kCFRunLoopDefaultMode);

        println!("Event tap enabled. Listening for mouse events...");
        // Run the run loop indefinitely.
        CFRunLoopRun();
    }
}
