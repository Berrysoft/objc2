use libc::{c_char, c_void};

use runtime::{Class, Object, Sel};

/// Types that have an Objective-C type encoding. For more information, see
/// Apple's documentation:
/// https://developer.apple.com/library/mac/documentation/Cocoa/Conceptual/ObjCRuntimeGuide/Articles/ocrtTypeEncodings.html
pub trait Encode {
    /// Return the Encoding for Self.
    fn code() -> &'static str;
}

impl Encode for i8 {
    fn code() -> &'static str { "c" }
}

impl Encode for i16 {
    fn code() -> &'static str { "s" }
}

impl Encode for i32 {
    fn code() -> &'static str { "i" }
}

impl Encode for i64 {
    fn code() -> &'static str { "q" }
}

impl Encode for isize {
    #[cfg(target_pointer_width = "32")]
    fn code() -> &'static str { "i" }

    #[cfg(target_pointer_width = "64")]
    fn code() -> &'static str { "q" }
}

impl Encode for u8 {
    fn code() -> &'static str { "C" }
}

impl Encode for u16 {
    fn code() -> &'static str { "S" }
}

impl Encode for u32 {
    fn code() -> &'static str { "I" }
}

impl Encode for u64 {
    fn code() -> &'static str { "Q" }
}

impl Encode for usize {
    #[cfg(target_pointer_width = "32")]
    fn code() -> &'static str { "I" }

    #[cfg(target_pointer_width = "64")]
    fn code() -> &'static str { "Q" }
}

impl Encode for f32 {
    fn code() -> &'static str { "f" }
}

impl Encode for f64 {
    fn code() -> &'static str { "d" }
}

impl Encode for bool {
    fn code() -> &'static str { "B" }
}

impl Encode for () {
    fn code() -> &'static str { "v" }
}

impl Encode for *mut c_char {
    fn code() -> &'static str { "*" }
}

impl Encode for *const c_char {
    fn code() -> &'static str { "r*" }
}

impl Encode for *mut c_void {
    fn code() -> &'static str { "^v" }
}

impl Encode for *const c_void {
    fn code() -> &'static str { "r^v" }
}

impl Encode for Sel {
    fn code() -> &'static str { ":" }
}

pub trait EncodePtr {
    fn ptr_code() -> &'static str;
}

impl<'a, T> Encode for &'a T where T: EncodePtr {
    fn code() -> &'static str { <T as EncodePtr>::ptr_code() }
}

impl<'a, T> Encode for &'a mut T where T: EncodePtr {
    fn code() -> &'static str { <T as EncodePtr>::ptr_code() }
}

impl<'a, T> Encode for Option<&'a T> where T: EncodePtr {
    fn code() -> &'static str { <T as EncodePtr>::ptr_code() }
}

impl<'a, T> Encode for Option<&'a mut T> where T: EncodePtr {
    fn code() -> &'static str { <T as EncodePtr>::ptr_code() }
}

impl<T> Encode for *const T where T: EncodePtr {
    fn code() -> &'static str { <T as EncodePtr>::ptr_code() }
}

impl<T> Encode for *mut T where T: EncodePtr {
    fn code() -> &'static str { <T as EncodePtr>::ptr_code() }
}

impl EncodePtr for Object {
    fn ptr_code() -> &'static str { "@" }
}

impl EncodePtr for Class {
    fn ptr_code() -> &'static str { "#" }
}

/// Returns the Objective-C type encoding for a type.
pub fn encode<T>() -> &'static str where T: Encode {
    <T as Encode>::code()
}

#[cfg(test)]
mod tests {
    use runtime::{Class, Object, Sel};
    use super::encode;

    #[test]
    fn test_encode() {
        assert!(encode::<u32>() == "I");
        assert!(encode::<()>() == "v");
        assert!(encode::<&Object>() == "@");
        assert!(encode::<*mut Object>() == "@");
        assert!(encode::<&Class>() == "#");
        assert!(encode::<Sel>() == ":");
    }
}
