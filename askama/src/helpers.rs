use std::fmt;
use std::iter::{Enumerate, Peekable};

use askama_escape::MarkupDisplay;

pub struct TemplateLoop<I>
where
    I: Iterator,
{
    iter: Peekable<Enumerate<I>>,
}

impl<I> TemplateLoop<I>
where
    I: Iterator,
{
    #[inline]
    pub fn new(iter: I) -> Self {
        TemplateLoop {
            iter: iter.enumerate().peekable(),
        }
    }
}

impl<I> Iterator for TemplateLoop<I>
where
    I: Iterator,
{
    type Item = (<I as Iterator>::Item, LoopItem);

    #[inline]
    fn next(&mut self) -> Option<(<I as Iterator>::Item, LoopItem)> {
        self.iter.next().map(|(index, item)| {
            (
                item,
                LoopItem {
                    index,
                    first: index == 0,
                    last: self.iter.peek().is_none(),
                },
            )
        })
    }
}

#[derive(Copy, Clone)]
pub struct LoopItem {
    pub index: usize,
    pub first: bool,
    pub last: bool,
}

pub trait MarkupDisplayWrap {
    type Result: fmt::Display;

    fn askama_new_unsafe(self) -> Self::Result;
}

impl<'a, E, T> MarkupDisplayWrap for &'a (&'a T, E)
where
    E: askama_escape::Escaper + Copy,
    T: fmt::Display,
{
    type Result = MarkupDisplay<E, &'a T>;

    #[inline]
    fn askama_new_unsafe(self) -> Self::Result {
        MarkupDisplay::new_unsafe(self.0, self.1)
    }
}

impl<'a, E, T> MarkupDisplayWrap for (&'a Option<T>, E)
where
    E: askama_escape::Escaper + Copy,
    T: fmt::Display,
{
    type Result = OptionMarkupDisplay<'a, E, T>;

    #[inline]
    fn askama_new_unsafe(self) -> Self::Result {
        OptionMarkupDisplay {
            value: self.0.as_ref(),
            escaper: self.1,
        }
    }
}

pub struct OptionMarkupDisplay<'a, E, T>
where
    E: askama_escape::Escaper + Copy,
    T: fmt::Display,
{
    value: Option<&'a T>,
    escaper: E,
}

impl<'a, E, T> fmt::Display for OptionMarkupDisplay<'a, E, T>
where
    E: askama_escape::Escaper + Copy,
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Some(value) => MarkupDisplay::new_unsafe(value, self.escaper).fmt(f),
            None => Ok(()),
        }
    }
}
