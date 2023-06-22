//! Ensure that `init` returns the same type as given from `alloc`.
use objc2::rc::{Allocated, Id};
use objc2::runtime::{NSObject, AnyObject};
use objc2::{class, msg_send_id};

fn main() {
    let cls = class!(NSObject);
    let obj: Option<Allocated<NSObject>> = unsafe { msg_send_id![cls, alloc] };

    let _: Id<AnyObject> = unsafe { msg_send_id![obj, init] };
}
