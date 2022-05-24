#![cfg_attr(feature = "cargo-clippy", allow(unused_parens))]
#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

use std::fmt;

pub use askama_escape::MarkupDisplay;

mod error;
pub use crate::error::{Error, Result};
pub mod filters;
pub mod helpers;

/// Main `Template` trait; implementations are generally derived
///
/// If you need an object-safe template, use [`DynTemplate`].
pub trait Template: fmt::Display {
    /// Helper method which allocates a new `String` and renders into it
    fn render(&self) -> Result<String> {
        let mut buf = String::with_capacity(Self::SIZE_HINT);
        self.render_into(&mut buf)?;
        Ok(buf)
    }

    /// Renders the template to the given `writer` fmt buffer
    fn render_into(&self, writer: &mut (impl std::fmt::Write + ?Sized)) -> Result<()>;

    /// Renders the template to the given `writer` io buffer
    #[inline]
    fn write_into(&self, writer: &mut (impl std::io::Write + ?Sized)) -> std::io::Result<()> {
        writer.write_fmt(format_args!("{}", self))
    }

    /// The template's extension, if provided
    const EXTENSION: Option<&'static str>;

    /// Provides a conservative estimate of the expanded length of the rendered template
    const SIZE_HINT: usize;

    /// The MIME type (Content-Type) of the data that gets rendered by this Template
    const MIME_TYPE: &'static str;
}

/// Object-safe wrapper trait around [`Template`] implementers
///
/// This trades reduced performance (mostly due to writing into `dyn Write`) for object safety.
pub trait DynTemplate {
    /// Helper method which allocates a new `String` and renders into it
    fn dyn_render(&self) -> Result<String>;

    /// Renders the template to the given `writer` fmt buffer
    fn dyn_render_into(&self, writer: &mut dyn std::fmt::Write) -> Result<()>;

    /// Renders the template to the given `writer` io buffer
    fn dyn_write_into(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()>;

    /// Helper function to inspect the template's extension
    fn extension(&self) -> Option<&'static str>;

    /// Provides a conservative estimate of the expanded length of the rendered template
    fn size_hint(&self) -> usize;

    /// The MIME type (Content-Type) of the data that gets rendered by this Template
    fn mime_type(&self) -> &'static str;
}

impl<T: Template> DynTemplate for T {
    fn dyn_render(&self) -> Result<String> {
        <Self as Template>::render(self)
    }

    fn dyn_render_into(&self, writer: &mut dyn std::fmt::Write) -> Result<()> {
        <Self as Template>::render_into(self, writer)
    }

    #[inline]
    fn dyn_write_into(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        writer.write_fmt(format_args!("{}", self))
    }

    fn extension(&self) -> Option<&'static str> {
        Self::EXTENSION
    }

    fn size_hint(&self) -> usize {
        Self::SIZE_HINT
    }

    fn mime_type(&self) -> &'static str {
        Self::MIME_TYPE
    }
}

impl fmt::Display for dyn DynTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.dyn_render_into(f).map_err(|_| ::std::fmt::Error {})
    }
}

#[cfg(test)]
#[allow(clippy::blacklisted_name)]
mod tests {
    use std::fmt;

    use super::*;
    use crate::{DynTemplate, Template};

    #[test]
    fn dyn_template() {
        struct Test;
        impl Template for Test {
            fn render_into(&self, writer: &mut (impl std::fmt::Write + ?Sized)) -> Result<()> {
                Ok(writer.write_str("test")?)
            }

            const EXTENSION: Option<&'static str> = Some("txt");

            const SIZE_HINT: usize = 4;

            const MIME_TYPE: &'static str = "text/plain; charset=utf-8";
        }

        impl fmt::Display for Test {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.render_into(f).map_err(|_| fmt::Error {})
            }
        }

        fn render(t: &dyn DynTemplate) -> String {
            t.dyn_render().unwrap()
        }

        let test = &Test as &dyn DynTemplate;

        assert_eq!(render(test), "test");

        assert_eq!(test.to_string(), "test");

        assert_eq!(format!("{}", test), "test");

        let mut vec = Vec::new();
        test.dyn_write_into(&mut vec).unwrap();
        assert_eq!(vec, vec![b't', b'e', b's', b't']);
    }
}
