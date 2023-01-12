#![cfg(feature = "Foundation_NSNumber")]
//! Note that due to limitations in Objective-C type encodings, it is not
//! possible to distinguish between an `NSNumber` created from [`bool`],
//! and one created from an [`i8`]/[`u8`]. You should use the getter
//! methods that fit your use-case instead!
//!
//! This does not implement [`Eq`] nor [`Ord`], since it may contain a
//! floating point value. Beware that the implementation of [`PartialEq`]
//! and [`PartialOrd`] does not properly handle NaNs either. Compare
//! [`NSNumber::encoding`] with [`Encoding::Float`] or
//! [`Encoding::Double`], and use [`NSNumber::as_f32`] or
//! [`NSNumber::as_f64`] to get the desired floating point value directly.
use core::cmp::Ordering;
use core::fmt;
use core::hash;
use core::panic::{RefUnwindSafe, UnwindSafe};

use objc2::encode::Encoding;

use crate::common::*;
use crate::Foundation::{CGFloat, NSNumber};

// SAFETY: `NSNumber` is just a wrapper around an integer/float/bool, and it
// is immutable.
unsafe impl Sync for NSNumber {}
unsafe impl Send for NSNumber {}

impl UnwindSafe for NSNumber {}
impl RefUnwindSafe for NSNumber {}

macro_rules! def_new_fn {
    {$(
        $(#[$($m:meta)*])*
        ($fn_name:ident($fn_inp:ty); $method_name:ident),
    )*} => {$(
        $(#[$($m)*])*
        pub fn $fn_name(val: $fn_inp) -> Id<Self, Shared> {
            Self::$method_name(val as _)
        }
    )*}
}

/// Creation methods.
impl NSNumber {
    def_new_fn! {
        (new_bool(bool); numberWithBool),
        (new_i8(i8); numberWithChar),
        (new_u8(u8); numberWithUnsignedChar),
        (new_i16(i16); numberWithShort),
        (new_u16(u16); numberWithUnsignedShort),
        (new_i32(i32); numberWithInt),
        (new_u32(u32); numberWithUnsignedInt),
        (new_i64(i64); numberWithLongLong),
        (new_u64(u64); numberWithUnsignedLongLong),
        (new_isize(isize); numberWithInteger),
        (new_usize(usize); numberWithUnsignedInteger),
        (new_f32(f32); numberWithFloat),
        (new_f64(f64); numberWithDouble),
    }

    #[inline]
    pub fn new_cgfloat(val: CGFloat) -> Id<Self, Shared> {
        #[cfg(target_pointer_width = "64")]
        {
            Self::new_f64(val)
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            Self::new_f32(val)
        }
    }
}

macro_rules! def_get_fn {
    {$(
        $(#[$($m:meta)*])*
        ($fn_name:ident -> $fn_ret:ty; $method_name:ident),
    )*} => {$(
        $(#[$($m)*])*
        pub fn $fn_name(&self) -> $fn_ret {
            self.$method_name() as _
        }
    )*}
}

/// Getter methods.
impl NSNumber {
    def_get_fn! {
        (as_bool -> bool; boolValue),
        (as_i8 -> i8; charValue),
        (as_u8 -> u8; unsignedCharValue),
        (as_i16 -> i16; shortValue),
        (as_u16 -> u16; unsignedShortValue),
        (as_i32 -> i32; intValue),
        (as_u32 -> u32; unsignedIntValue),
        (as_i64 -> i64; longLongValue),
        (as_u64 -> u64; unsignedLongLongValue),
        (as_isize -> isize; integerValue),
        (as_usize -> usize; unsignedIntegerValue),
        (as_f32 -> f32; floatValue),
        (as_f64 -> f64; doubleValue),
    }

    #[inline]
    pub fn as_cgfloat(&self) -> CGFloat {
        #[cfg(target_pointer_width = "64")]
        {
            self.as_f64()
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            self.as_f32()
        }
    }

    /// The Objective-C encoding of this `NSNumber`.
    ///
    /// This is guaranteed to return one of:
    /// - [`Encoding::Char`]
    /// - [`Encoding::UChar`]
    /// - [`Encoding::Short`]
    /// - [`Encoding::UShort`]
    /// - [`Encoding::Int`]
    /// - [`Encoding::UInt`]
    /// - [`Encoding::Long`]
    /// - [`Encoding::ULong`]
    /// - [`Encoding::LongLong`]
    /// - [`Encoding::ULongLong`]
    /// - [`Encoding::Float`]
    /// - [`Encoding::Double`]
    ///
    ///
    /// # Examples
    ///
    /// Convert an `NSNumber` to/from an enumeration describing the different
    /// number properties.
    ///
    /// ```
    /// use icrate::Foundation::NSNumber;
    /// use icrate::objc2::Encoding;
    /// use icrate::objc2::rc::{Id, Shared};
    ///
    /// // Note: `bool` would convert to either `Signed` or `Unsigned`,
    /// // depending on platform
    /// #[derive(Copy, Clone)]
    /// pub enum Number {
    ///     Signed(i64),
    ///     Unsigned(u64),
    ///     Floating(f64),
    /// }
    ///
    /// impl Number {
    ///     fn into_nsnumber(self) -> Id<NSNumber, Shared> {
    ///         match self {
    ///             Self::Signed(val) => NSNumber::new_i64(val),
    ///             Self::Unsigned(val) => NSNumber::new_u64(val),
    ///             Self::Floating(val) => NSNumber::new_f64(val),
    ///         }
    ///     }
    /// }
    ///
    /// impl From<&NSNumber> for Number {
    ///     fn from(n: &NSNumber) -> Self {
    ///         match n.encoding() {
    ///             Encoding::Char
    ///             | Encoding::Short
    ///             | Encoding::Int
    ///             | Encoding::Long
    ///             | Encoding::LongLong => Self::Signed(n.as_i64()),
    ///             Encoding::UChar
    ///             | Encoding::UShort
    ///             | Encoding::UInt
    ///             | Encoding::ULong
    ///             | Encoding::ULongLong => Self::Unsigned(n.as_u64()),
    ///             Encoding::Float
    ///             | Encoding::Double => Self::Floating(n.as_f64()),
    ///             _ => unreachable!(),
    ///         }
    ///     }
    /// }
    /// ```
    pub fn encoding(&self) -> Encoding {
        // Use NSValue::encoding
        let enc = (**self)
            .encoding()
            .expect("NSNumber must have an encoding!");

        // Guaranteed under "Subclassing Notes"
        // <https://developer.apple.com/documentation/foundation/nsnumber?language=objc#1776615>
        match enc {
            "c" => Encoding::Char,
            "C" => Encoding::UChar,
            "s" => Encoding::Short,
            "S" => Encoding::UShort,
            "i" => Encoding::Int,
            "I" => Encoding::UInt,
            "l" => Encoding::Long,
            "L" => Encoding::ULong,
            "q" => Encoding::LongLong,
            "Q" => Encoding::ULongLong,
            "f" => Encoding::Float,
            "d" => Encoding::Double,
            _ => unreachable!("invalid encoding for NSNumber"),
        }
    }
}

impl hash::Hash for NSNumber {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

/// Beware: This uses the Objective-C method "isEqualToNumber:", which has
/// different floating point NaN semantics than Rust!
impl PartialEq for NSNumber {
    #[doc(alias = "isEqualToNumber:")]
    fn eq(&self, other: &Self) -> bool {
        // Use isEqualToNumber: instaed of isEqual: since it is faster
        self.isEqualToNumber(other)
    }
}

/// Beware: This uses the Objective-C method "compare:", which has different
/// floating point NaN semantics than Rust!
impl PartialOrd for NSNumber {
    #[doc(alias = "compare:")]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Use Objective-C semantics for comparison
        Some(self.compare(other).into())
    }
}

#[cfg(feature = "Foundation_NSString")]
impl fmt::Display for NSNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.stringValue(), f)
    }
}
