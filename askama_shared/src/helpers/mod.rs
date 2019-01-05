use std::iter::Peekable;

pub struct Enumerate<I>
where
    I: Iterator,
{
    iter: Peekable<I>,
    count: usize,
}

impl<I> Iterator for Enumerate<I>
where
    I: Iterator,
{
    type Item = (usize, bool, <I as Iterator>::Item);

    #[inline]
    fn next(&mut self) -> Option<(usize, bool, <I as Iterator>::Item)> {
        self.iter.next().map(|a| {
            let last = self.iter.peek().is_none();
            let ret = (self.count, last, a);
            // Possible undefined overflow.
            self.count += 1;
            ret
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }
}

#[inline]
pub fn enumerate<I>(iter: I) -> Enumerate<I>
where
    I: Iterator,
{
    Enumerate {
        iter: iter.peekable(),
        count: 0,
    }
}
