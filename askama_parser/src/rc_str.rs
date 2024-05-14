use std::{borrow, cmp, fmt, hash, iter, mem, ops, rc, slice, str};

#[derive(Clone, Default)]
pub struct RcStr {
    value: &'static str,
    _rc: rc::Rc<String>,
}

impl ops::Deref for RcStr {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl RcStr {
    #[inline]
    pub fn as_str(&self) -> &str {
        self.value
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.value.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn substr(&self, range: impl slice::SliceIndex<str, Output = str>) -> Self {
        Self {
            value: self.value.get(range).unwrap(),
            _rc: self._rc.clone(),
        }
    }

    pub fn split_at(&self, mid: usize) -> (Self, Self) {
        let (a, b) = self.value.split_at(mid);
        let a = Self {
            value: a,
            _rc: self._rc.clone(),
        };
        let b = Self {
            value: b,
            _rc: self._rc.clone(),
        };
        (a, b)
    }

    pub fn trim_start_matches(&self, p: impl Fn(char) -> bool) -> Self {
        Self {
            value: self.value.trim_start_matches(p),
            _rc: self._rc.clone(),
        }
    }

    pub fn trim_end_matches(&self, p: impl Fn(char) -> bool) -> Self {
        Self {
            value: self.value.trim_end_matches(p),
            _rc: self._rc.clone(),
        }
    }

    #[inline]
    fn split_at_reversed(&self, mid: usize) -> (Self, Self) {
        let (a, b) = self.split_at(mid);
        (b, a)
    }
}

impl fmt::Debug for RcStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl fmt::Display for RcStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl hash::Hash for RcStr {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}

impl borrow::Borrow<str> for RcStr {
    #[inline]
    fn borrow(&self) -> &str {
        self.value
    }
}

impl cmp::PartialEq for RcStr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl cmp::Eq for RcStr {}

impl cmp::PartialEq<str> for RcStr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.value == other
    }
}

impl cmp::PartialEq<&str> for RcStr {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.value == *other
    }
}

impl cmp::PartialOrd for RcStr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.value.partial_cmp(other.value)
    }
}

impl cmp::PartialOrd<str> for RcStr {
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        self.value.partial_cmp(other)
    }
}

impl cmp::PartialOrd<&str> for RcStr {
    #[inline]
    fn partial_cmp(&self, other: &&str) -> Option<cmp::Ordering> {
        self.value.partial_cmp(other)
    }
}

impl From<String> for RcStr {
    fn from(rc: String) -> Self {
        let rc = rc::Rc::new(rc);
        let value = unsafe { mem::transmute::<&str, &str>(rc.as_str()) };
        RcStr { value, _rc: rc }
    }
}

impl AsRef<str> for RcStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.value
    }
}

impl AsRef<[u8]> for RcStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.value.as_bytes()
    }
}

impl<R> nom::Slice<R> for RcStr
where
    for<'x> &'x str: nom::Slice<R>,
{
    fn slice(&self, range: R) -> Self {
        Self {
            value: self.value.slice(range),
            _rc: self._rc.clone(),
        }
    }
}

impl nom::AsBytes for RcStr {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self.value.as_bytes()
    }
}

impl<T: AsRef<str>> nom::Compare<T> for RcStr {
    #[inline]
    fn compare(&self, t: T) -> nom::CompareResult {
        self.value.compare(t.as_ref())
    }

    #[inline]
    fn compare_no_case(&self, t: T) -> nom::CompareResult {
        self.value.compare_no_case(t.as_ref())
    }
}

impl<T: AsRef<str>> nom::FindSubstring<T> for RcStr {
    #[inline]
    fn find_substring(&self, substr: T) -> Option<usize> {
        self.value.find_substring(substr.as_ref())
    }
}

impl nom::FindToken<u8> for RcStr {
    #[inline]
    fn find_token(&self, token: u8) -> bool {
        self.value.find_token(token)
    }
}

impl nom::HexDisplay for RcStr {
    #[inline]
    fn to_hex(&self, chunk_size: usize) -> String {
        self.value.to_hex(chunk_size)
    }

    #[inline]
    fn to_hex_from(&self, chunk_size: usize, from: usize) -> String {
        self.value.to_hex_from(chunk_size, from)
    }
}

impl nom::InputIter for RcStr {
    type Item = char;

    type Iter = RcCharIndices;

    type IterElem = RcChars;

    #[inline]
    fn iter_indices(&self) -> Self::Iter {
        Self::Iter::from(self)
    }

    #[inline]
    fn iter_elements(&self) -> Self::IterElem {
        Self::IterElem::from(self)
    }

    #[inline]
    fn position<P: Fn(Self::Item) -> bool>(&self, predicate: P) -> Option<usize> {
        self.value.position(predicate)
    }

    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        self.value.slice_index(count)
    }
}

impl nom::InputLength for RcStr {
    #[inline]
    fn input_len(&self) -> usize {
        self.value.input_len()
    }
}

impl nom::InputTake for RcStr {
    fn take(&self, count: usize) -> Self {
        Self {
            value: self.value.take(count),
            _rc: self._rc.clone(),
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (a, b) = self.value.take_split(count);
        let a = Self {
            value: a,
            _rc: self._rc.clone(),
        };
        let b = Self {
            value: b,
            _rc: self._rc.clone(),
        };
        (a, b)
    }
}

impl nom::InputTakeAtPosition for RcStr {
    type Item = char;

    fn split_at_position<P, E>(&self, predicate: P) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
        E: nom::error::ParseError<Self>,
    {
        match self.value.find(predicate) {
            Some(i) => Ok(self.split_at_reversed(i)),
            None => Err(nom::Err::Incomplete(nom::Needed::new(1))),
        }
    }

    fn split_at_position1<P, E>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
        E: nom::error::ParseError<Self>,
    {
        match self.value.find(predicate) {
            Some(0) => Err(nom::Err::Error(E::from_error_kind(self.clone(), e))),
            Some(i) => Ok(self.split_at_reversed(i)),
            None => Err(nom::Err::Incomplete(nom::Needed::new(1))),
        }
    }

    fn split_at_position_complete<P, E>(&self, predicate: P) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
        E: nom::error::ParseError<Self>,
    {
        match self.value.find(predicate) {
            Some(i) => Ok(self.split_at_reversed(i)),
            None => Ok(self.split_at_reversed(self.value.len())),
        }
    }

    fn split_at_position1_complete<P, E>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
        E: nom::error::ParseError<Self>,
    {
        match self.value.find(predicate) {
            Some(0) => Err(nom::Err::Error(E::from_error_kind(self.clone(), e))),
            Some(i) => Ok(self.split_at_reversed(i)),
            None => {
                if self.value.is_empty() {
                    Err(nom::Err::Error(E::from_error_kind(self.clone(), e)))
                } else {
                    Ok(self.split_at_reversed(self.value.len()))
                }
            }
        }
    }
}

impl nom::Offset for RcStr {
    fn offset(&self, second: &Self) -> usize {
        self.value.offset(second.value)
    }
}

#[derive(Debug, Clone)]
pub struct RcChars {
    iter: str::Chars<'static>,
    _rc: rc::Rc<String>,
}

impl From<&RcStr> for RcChars {
    fn from(value: &RcStr) -> Self {
        Self {
            iter: value.value.chars(),
            _rc: value._rc.clone(),
        }
    }
}

impl iter::FusedIterator for RcChars {}

impl DoubleEndedIterator for RcChars {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl Iterator for RcChars {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Debug, Clone)]
pub struct RcCharIndices {
    iter: str::CharIndices<'static>,
    _rc: rc::Rc<String>,
}

impl From<&RcStr> for RcCharIndices {
    fn from(value: &RcStr) -> Self {
        Self {
            iter: value.value.char_indices(),
            _rc: value._rc.clone(),
        }
    }
}

impl iter::FusedIterator for RcCharIndices {}

impl DoubleEndedIterator for RcCharIndices {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl Iterator for RcCharIndices {
    type Item = (usize, char);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
