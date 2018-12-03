#![cfg_attr(askama_nightly, feature(specialization))]

#[macro_use]
extern crate cfg_if;
extern crate v_htmlescape;

use std::fmt::{self, Display, Formatter};
use std::str;

use v_htmlescape::Escape;

#[derive(Debug, PartialEq)]
pub enum MarkupDisplay<T>
where
    T: Display,
{
    Safe(T),
    Unsafe(T),
}

cfg_if! {
    if #[cfg(askama_nightly)] {

        pub trait ANum {}

        macro_rules! trait_impl {
            ($name:ident for $($t:ty)*) => ($(
                impl $name for $t {
                }
            )*)
        }

        // TODO: MarkupDisplay::from parameter, check number of borrow's
        trait_impl!(ANum for usize u8 u16 u32 u64 isize i8 i16 i32 i64);
        trait_impl!(ANum for &usize &u8 &u16 &u32 &u64 &isize &i8 &i16 &i32 &i64);
        trait_impl!(ANum for &&usize &&u8 &&u16 &&u32 &&u64 &&isize &&i8 &&i16 &&i32 &&i64);

        #[cfg(has_i128)]
        trait_impl!(ANum for u128 i128);
        #[cfg(has_i128)]
        trait_impl!(ANum for &u128 &i128);
        #[cfg(has_i128)]
        trait_impl!(ANum for &&u128 &&i128);

        impl<T> From<T> for MarkupDisplay<T>
        where
            T: Display,
        {
            default fn from(t: T) -> MarkupDisplay<T> {
                MarkupDisplay::Unsafe(t)
            }
        }

        impl<T> From<T> for MarkupDisplay<T>
        where
            T: Display + ANum,
        {
            fn from(t: T) -> MarkupDisplay<T> {
                MarkupDisplay::Safe(t)
            }
        }

        impl<T> Display for MarkupDisplay<T>
        where
            T: Display,
        {
            default fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                match *self {
                    MarkupDisplay::Unsafe(ref t) => escape(&t.to_string()).fmt(f),
                    MarkupDisplay::Safe(ref t) => t.fmt(f),
                }
            }
        }

        pub trait IsStr {
            fn cow(&self) -> &str;
        }

        macro_rules! string_trait_impl {
            ($($t:ty)*) => ($(
                impl IsStr for $t {
                    #[inline]
                    fn cow(&self) -> &str {
                        &self
                    }
                }
            )*)
        }

        string_trait_impl!(String &String &&String);

        macro_rules! str_trait_impl {
            ($($t:ty)*) => ($(
                impl IsStr for $t {
                    #[inline]
                    fn cow(&self) -> &str {
                        self
                    }
                }
            )*)
        }

        str_trait_impl!(&str &&str &&&str);

        impl<T> Display for MarkupDisplay<T>
        where
            T: Display + IsStr,
        {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                match *self {
                    MarkupDisplay::Unsafe(ref t) => escape(t.cow()).fmt(f),
                    MarkupDisplay::Safe(ref t) => t.fmt(f),
                }
            }
        }
    } else {

        impl<T> From<T> for MarkupDisplay<T>
        where
            T: Display,
        {
            fn from(t: T) -> MarkupDisplay<T> {
                MarkupDisplay::Unsafe(t)
            }
        }

        impl<T> Display for MarkupDisplay<T>
        where
            T: Display,
        {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                match *self {
                    MarkupDisplay::Unsafe(ref t) => escape(&t.to_string()).fmt(f),
                    MarkupDisplay::Safe(ref t) => t.fmt(f),
                }
            }
        }
    }
}

impl<T> MarkupDisplay<T>
where
    T: Display,
{
    pub fn mark_safe(self) -> MarkupDisplay<T> {
        match self {
            MarkupDisplay::Unsafe(t) => MarkupDisplay::Safe(t),
            _ => self,
        }
    }
}

#[inline]
pub fn escape(s: &str) -> Escape {
    Escape::new(s.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape() {
        assert_eq!(escape("").to_string(), "");
        assert_eq!(escape("<&>").to_string(), "&lt;&amp;&gt;");
        assert_eq!(escape("bla&").to_string(), "bla&amp;");
        assert_eq!(escape("<foo").to_string(), "&lt;foo");
        assert_eq!(escape("bla&h").to_string(), "bla&amp;h");
    }
}
