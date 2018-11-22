#[macro_use]
extern crate cfg_if;

use std::fmt::{self, Display, Formatter};
use std::str;

#[derive(Debug, PartialEq)]
pub enum MarkupDisplay<T>
where
    T: Display,
{
    Safe(T),
    Unsafe(T),
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

pub fn escape(s: &str) -> Escaped {
    Escaped {
        bytes: s.as_bytes(),
    }
}

pub struct Escaped<'a> {
    bytes: &'a [u8],
}

impl<'a> Display for Escaped<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        _imp(self.bytes, fmt)
    }
}

cfg_if! {
    if #[cfg(all(target_arch = "x86_64", not(target_os = "windows"), askama_runtime_simd))] {

        use std::arch::x86_64::*;
        use std::mem::{self, size_of};
        use std::sync::atomic::{AtomicUsize, Ordering};

        #[inline(always)]
        fn _imp(bytes: &[u8], fmt: &mut Formatter) -> fmt::Result {
            // https://github.com/BurntSushi/rust-memchr/blob/master/src/x86/mod.rs#L9-L29
            static mut FN: fn(bytes: &[u8], fmt: &mut Formatter) -> fmt::Result = detect;

            fn detect(bytes: &[u8], fmt: &mut Formatter) -> fmt::Result {
                let fun = if cfg!(askama_runtime_avx) && is_x86_feature_detected!("avx2") {
                    _avx_escape as usize
                } else if cfg!(askama_runtime_sse) && is_x86_feature_detected!("sse4.2") {
                    _sse_escape as usize
                } else {
                    _escape as usize
                };

                let slot = unsafe { &*(&FN as *const _ as *const AtomicUsize) };
                slot.store(fun as usize, Ordering::Relaxed);
                unsafe {
                    mem::transmute::<usize, fn(bytes: &[u8], fmt: &mut Formatter) -> fmt::Result>(fun)(bytes, fmt)
                }
            }

            unsafe {
                let slot = &*(&FN as *const _ as * const AtomicUsize);
                let fun = slot.load(Ordering::Relaxed);
                mem::transmute::<usize, fn(bytes: &[u8], fmt: &mut Formatter) -> fmt::Result>(fun)(bytes, fmt)
            }
        }

        // Subtract `b` from `a` and return the difference. `a` should be greater than
        // or equal to `b`.
        #[inline(always)]
        fn sub(a: *const u8, b: *const u8) -> usize {
            debug_assert!(b <= a);
            (a as usize) - (b as usize)
        }
    } else {

        #[inline(always)]
        fn _imp(bytes: &[u8], fmt: &mut Formatter) -> fmt::Result {
            _escape(bytes, fmt)
        }
    }
}

// Main instruction in escape
// Test 'start' iterator in current position,
// write slice since start to current position - 1,
// escape byte with quote and
// set 'start' iterator with next position
macro_rules! escape_body {
    ($i:expr, $start:ident, $fmt:ident, $bytes:ident, $quote:expr) => {{
        if $start < $i {
            #[allow(unused_unsafe)]
            $fmt.write_str(unsafe { str::from_utf8_unchecked(&$bytes[$start..$i]) })?;
        }
        $fmt.write_str($quote)?;
        $start = $i + 1;
    }};
}

// Select between pairs bytes - quote to call the callback
macro_rules! bodies {
    ($i:expr, $b:expr, $start:ident, $fmt:ident, $bytes:ident, $callback:ident) => {
        match $b {
            b'<' => $callback!($i, $start, $fmt, $bytes, "&lt;"),
            b'>' => $callback!($i, $start, $fmt, $bytes, "&gt;"),
            b'&' => $callback!($i, $start, $fmt, $bytes, "&amp;"),
            b'"' => $callback!($i, $start, $fmt, $bytes, "&quot;"),
            b'\'' => $callback!($i, $start, $fmt, $bytes, "&#x27;"),
            b'/' => $callback!($i, $start, $fmt, $bytes, "&#x2f;"),
            _ => (),
        }
    };
}

// Wrap the body of the escape over the body of the mask
// Resolve expression and do the escape body
#[allow(unused_macros)]
macro_rules! mask_body {
    ($i:expr, $start:ident, $fmt:ident, $bytes:ident, $quote:expr) => {{
        let i = $i;
        escape_body!(i, $start, $fmt, $bytes, $quote);
    }};
}

// Format bytes in the mask that starts in the current pointer
#[allow(unused_macros)]
macro_rules! mask_bodies {
    ($mask:ident, $at:ident, $cur:ident, $ptr:ident, $start:ident, $fmt:ident, $bytes:ident) => {
        bodies!($at + $cur, *$ptr.add($cur), $start, $fmt, $bytes, mask_body);

        $mask ^= 1 << $cur;
        if $mask == 0 {
            break;
        }

        $cur = $mask.trailing_zeros() as usize;
    };
}

// Write a mask
// Select starts mask byte pointer and current position to deep find.
// The main loop break when mask == 0
#[allow(unused_macros)]
macro_rules! write_mask {
    ($mask:ident, $ptr:ident, $start_ptr:ident, $start:ident, $fmt:ident, $bytes:ident) => {{
        let at = sub($ptr, $start_ptr);
        let mut cur = $mask.trailing_zeros() as usize;

        loop {
            mask_bodies!($mask, at, cur, $ptr, $start, $fmt, $bytes);
        }

        debug_assert_eq!(at, sub($ptr, $start_ptr))
    }};
}

/// Scalar html escape
fn _escape(bytes: &[u8], fmt: &mut Formatter) -> fmt::Result {
    let mut start = 0;

    for (i, b) in bytes.iter().enumerate() {
        if b.wrapping_sub(FLAG_BELOW) <= LEN {
            bodies!(i, *b, start, fmt, bytes, escape_body);
        }
    }

    fmt.write_str(unsafe { str::from_utf8_unchecked(&bytes[start..]) })?;

    Ok(())
}

#[cfg(all(
    target_arch = "x86_64",
    not(target_os = "windows"),
    askama_runtime_simd,
    askama_runtime_avx
))]
#[target_feature(enable = "avx2")]
unsafe fn _avx_escape(bytes: &[u8], fmt: &mut Formatter) -> fmt::Result {
    const VECTOR_SIZE: usize = size_of::<__m256i>();
    const VECTOR_ALIGN: usize = VECTOR_SIZE - 1;
    const LOOP_SIZE: usize = 4 * VECTOR_SIZE;

    let v_flag = _mm256_set1_epi8((LEN + 1) as i8);
    let v_flag_below = _mm256_set1_epi8(FLAG_BELOW as i8);

    let len = bytes.len();
    let start_ptr = bytes.as_ptr();
    let mut ptr = start_ptr;
    let mut start = 0;

    // Write a sliced mask
    macro_rules! write_forward {
        ($mask: ident, $align:ident) => {{
            if $mask != 0 {
                let at = sub(ptr, start_ptr);
                let mut cur = $mask.trailing_zeros() as usize;

                while cur < $align {
                    mask_bodies!($mask, at, cur, ptr, start, fmt, bytes);
                }

                debug_assert_eq!(at, sub(ptr, start_ptr))
            }
        }};
    }

    if len < VECTOR_SIZE {
        let a = _mm256_loadu_si256(ptr as *const __m256i);
        let cmp = _mm256_cmpgt_epi8(v_flag, _mm256_sub_epi8(a, v_flag_below));
        let mut mask = _mm256_movemask_epi8(cmp);

        write_forward!(mask, len);
    } else {
        let end_ptr = bytes[len..].as_ptr();

        {
            let align = (VECTOR_SIZE - (start_ptr as usize & VECTOR_ALIGN)) & VECTOR_ALIGN;
            if 0 < align {
                let a = _mm256_loadu_si256(ptr as *const __m256i);
                let cmp = _mm256_cmpgt_epi8(v_flag, _mm256_sub_epi8(a, v_flag_below));
                let mut mask = _mm256_movemask_epi8(cmp);

                write_forward!(mask, align);
                ptr = ptr.add(align);

                debug_assert!(start <= sub(ptr, start_ptr));
            }
        }

        debug_assert!(start_ptr <= ptr && start_ptr <= end_ptr.sub(VECTOR_SIZE));

        if LOOP_SIZE <= len {
            // Main loop 128 bytes, need aligned ptr at VECTOR_SIZE
            while ptr <= end_ptr.sub(LOOP_SIZE) {
                // Need aligned
                debug_assert_eq!(0, (ptr as usize) % VECTOR_SIZE);

                let a = _mm256_load_si256(ptr as *const __m256i);
                let b = _mm256_load_si256(ptr.add(VECTOR_SIZE) as *const __m256i);
                let c = _mm256_load_si256(ptr.add(VECTOR_SIZE * 2) as *const __m256i);
                let d = _mm256_load_si256(ptr.add(VECTOR_SIZE * 3) as *const __m256i);
                let cmp_a = _mm256_cmpgt_epi8(v_flag, _mm256_sub_epi8(a, v_flag_below));
                let cmp_b = _mm256_cmpgt_epi8(v_flag, _mm256_sub_epi8(b, v_flag_below));
                let cmp_c = _mm256_cmpgt_epi8(v_flag, _mm256_sub_epi8(c, v_flag_below));
                let cmp_d = _mm256_cmpgt_epi8(v_flag, _mm256_sub_epi8(d, v_flag_below));
                let or1 = _mm256_or_si256(cmp_a, cmp_b);
                let or2 = _mm256_or_si256(cmp_c, cmp_d);

                // Adjust the four masks in two from right to left.
                if _mm256_movemask_epi8(_mm256_or_si256(or1, or2)) != 0 {
                    let mut mask = _mm256_movemask_epi8(cmp_a) as i64
                        | (_mm256_movemask_epi8(cmp_b) as i64) << VECTOR_SIZE;

                    if mask != 0 {
                        write_mask!(mask, ptr, start_ptr, start, fmt, bytes);
                    }
                    let ptr = ptr.add(VECTOR_SIZE + VECTOR_SIZE);

                    mask = _mm256_movemask_epi8(cmp_b) as i64
                        | (_mm256_movemask_epi8(cmp_c) as i64) << VECTOR_SIZE;

                    if mask != 0 {
                        write_mask!(mask, ptr, start_ptr, start, fmt, bytes);
                    }
                }

                ptr = ptr.add(LOOP_SIZE);

                debug_assert!(start <= sub(ptr, start_ptr));
            }
        }

        while ptr <= end_ptr.sub(VECTOR_SIZE) {
            // Need aligned
            debug_assert_eq!(0, (ptr as usize) % VECTOR_SIZE);

            let a = _mm256_load_si256(ptr as *const __m256i);
            let cmp = _mm256_cmpgt_epi8(v_flag, _mm256_sub_epi8(a, v_flag_below));
            let mut mask = _mm256_movemask_epi8(cmp);

            if mask != 0 {
                write_mask!(mask, ptr, start_ptr, start, fmt, bytes);
            }
            ptr = ptr.add(VECTOR_SIZE);

            debug_assert!(start <= sub(ptr, start_ptr));
        }

        debug_assert!(end_ptr.sub(VECTOR_SIZE) < ptr);

        if ptr < end_ptr {
            // Need aligned
            debug_assert_eq!(0, (ptr as usize) % VECTOR_SIZE);

            let a = _mm256_load_si256(ptr as *const __m256i);
            let cmp = _mm256_cmpgt_epi8(v_flag, _mm256_sub_epi8(a, v_flag_below));
            let mut mask = _mm256_movemask_epi8(cmp);
            let end = sub(end_ptr, ptr);

            write_forward!(mask, end);
        }
    }

    // Write since start to the end of the slice
    debug_assert!(start <= len);
    if start < len {
        fmt.write_str(str::from_utf8_unchecked(&bytes[start..len]))?;
    }

    Ok(())
}

#[cfg(all(
    target_arch = "x86_64",
    not(target_os = "windows"),
    askama_runtime_simd,
    askama_runtime_sse
))]
#[target_feature(enable = "sse4.2")]
unsafe fn _sse_escape(bytes: &[u8], fmt: &mut Formatter) -> fmt::Result {
    const VECTOR_SIZE: usize = size_of::<__m128i>();
    const VECTOR_ALIGN: usize = VECTOR_SIZE - 1;
    const LOOP_SIZE: usize = 4 * VECTOR_SIZE;
    const NEEDLE_LEN: i32 = 6;

    let needle = _mm_setr_epi8(
        b'<' as i8, b'>' as i8, b'&' as i8, b'"' as i8,
        b'\'' as i8, b'/' as i8, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0,
    );

    let len = bytes.len();
    let start_ptr = bytes.as_ptr();
    let mut ptr = start_ptr;
    let mut start = 0;

    if len < VECTOR_SIZE {
        let a = _mm_loadu_si128(ptr as *const __m128i);
        let cmp = _mm_cmpestrm(needle, NEEDLE_LEN, a, len as i32, 0);
        let mut mask = _mm_extract_epi16(cmp, 0) as i16;

        // No need write forward because I specified string size in
        // compare instruction
        if mask != 0 {
            write_mask!(mask, ptr, start_ptr, start, fmt, bytes);
        }
    } else {
        let end_ptr = bytes[len..].as_ptr();

        {
            let align = (VECTOR_SIZE - (start_ptr as usize & VECTOR_ALIGN)) & VECTOR_ALIGN;
            if 0 < align {
                let a = _mm_loadu_si128(ptr as *const __m128i);
                let cmp = _mm_cmpestrm(needle, NEEDLE_LEN, a, align as i32, 0);
                let mut mask = _mm_extract_epi16(cmp, 0) as i16;

                if mask != 0 {
                    write_mask!(mask, ptr, start_ptr, start, fmt, bytes);
                }
                ptr = ptr.add(align);

                debug_assert!(start <= sub(ptr, start_ptr));
            }
        }

        if LOOP_SIZE <= len {
            // Main loop 64 bytes, need aligned ptr at VECTOR_SIZE
            while ptr <= end_ptr.sub(LOOP_SIZE) {
                // Need aligned
                debug_assert_eq!(0, (ptr as usize) % VECTOR_SIZE);

                let a = _mm_load_si128(ptr as *const __m128i);
                let b = _mm_load_si128(ptr.add(VECTOR_SIZE) as *const __m128i);
                let c = _mm_load_si128(ptr.add(VECTOR_SIZE * 2) as *const __m128i);
                let d = _mm_load_si128(ptr.add(VECTOR_SIZE * 3) as *const __m128i);
                let cmp_a = _mm_cmpestrm(needle, NEEDLE_LEN, a, VECTOR_SIZE as i32, 0);
                let cmp_b = _mm_cmpestrm(needle, NEEDLE_LEN, b, VECTOR_SIZE as i32, 0);
                let cmp_c = _mm_cmpestrm(needle, NEEDLE_LEN, c, VECTOR_SIZE as i32, 0);
                let cmp_d = _mm_cmpestrm(needle, NEEDLE_LEN, d, VECTOR_SIZE as i32, 0);
                let or1 = _mm_or_si128(cmp_a, cmp_b);
                let or2 = _mm_or_si128(cmp_c, cmp_d);

                // Adjust the four masks in one from right to left.
                if _mm_extract_epi16(_mm_or_si128(or1, or2), 0) != 0 {
                    let mut mask = _mm_extract_epi16(cmp_a, 0) as i64
                        | (_mm_extract_epi16(cmp_b, 0) as i64) << VECTOR_SIZE
                        | (_mm_extract_epi16(cmp_c, 0) as i64) << VECTOR_SIZE * 2
                        | (_mm_extract_epi16(cmp_d, 0) as i64) << VECTOR_SIZE * 3;

                    write_mask!(mask, ptr, start_ptr, start, fmt, bytes);
                }

                ptr = ptr.add(LOOP_SIZE);

                debug_assert!(start <= sub(ptr, start_ptr));
            }
        }

        while ptr <= end_ptr.sub(VECTOR_SIZE) {
            // Need aligned
            debug_assert_eq!(0, (ptr as usize) % VECTOR_SIZE);

            let a = _mm_load_si128(ptr as *const __m128i);
            let cmp = _mm_cmpestrm(needle, NEEDLE_LEN, a, VECTOR_SIZE as i32, 0);
            let mut mask = _mm_extract_epi16(cmp, 0) as i16;

            if mask != 0 {
                write_mask!(mask, ptr, start_ptr, start, fmt, bytes);
            }
            ptr = ptr.add(VECTOR_SIZE);

            debug_assert!(start <= sub(ptr, start_ptr));
        }

        debug_assert!(end_ptr.sub(VECTOR_SIZE) < ptr);

        if ptr < end_ptr {
            // Need aligned
            debug_assert_eq!(0, (ptr as usize) % VECTOR_SIZE);

            let end = sub(end_ptr, ptr);
            let a = _mm_load_si128(ptr as *const __m128i);
            let cmp = _mm_cmpestrm(needle, NEEDLE_LEN, a, end as i32, 0);
            let mut mask = _mm_extract_epi16(cmp, 0) as i16;

            // No need write forward because I specified string size in
            // compare instruction
            if mask != 0 {
                write_mask!(mask, ptr, start_ptr, start, fmt, bytes);
            }
        }
    }

    // Write since start to the end of the slice
    debug_assert!(start <= len);
    if start < len {
        fmt.write_str(str::from_utf8_unchecked(&bytes[start..len]))?;
    }

    Ok(())
}

// Defining character interval from ASCII table to create bit masks from slice to be escaped
const LEN: u8 = b'>' - b'"';
const FLAG_BELOW: u8 = b'"';

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape() {
        let escapes = "<>&\"'/";
        let escaped = "&lt;&gt;&amp;&quot;&#x27;&#x2f;";
        let string_long: &str = &"foobar".repeat(1024);

        assert_eq!(escape("").to_string(), "");
        assert_eq!(escape("<&>").to_string(), "&lt;&amp;&gt;");
        assert_eq!(escape("bar&").to_string(), "bar&amp;");
        assert_eq!(escape("<foo").to_string(), "&lt;foo");
        assert_eq!(escape("bar&h").to_string(), "bar&amp;h");
        assert_eq!(
            escape("// my <html> is \"unsafe\" & should be 'escaped'").to_string(),
            "&#x2f;&#x2f; my &lt;html&gt; is &quot;unsafe&quot; &amp; \
             should be &#x27;escaped&#x27;"
        );
        assert_eq!(escape(&"<".repeat(16)).to_string(), "&lt;".repeat(16));
        assert_eq!(escape(&"<".repeat(32)).to_string(), "&lt;".repeat(32));
        assert_eq!(escape(&"<".repeat(64)).to_string(), "&lt;".repeat(64));
        assert_eq!(escape(&"<".repeat(128)).to_string(), "&lt;".repeat(128));
        assert_eq!(escape(&"<".repeat(1024)).to_string(), "&lt;".repeat(1024));
        assert_eq!(escape(&"<".repeat(129)).to_string(), "&lt;".repeat(129));
        assert_eq!(
            escape(&"<".repeat(128 * 2 - 1)).to_string(),
            "&lt;".repeat(128 * 2 - 1)
        );
        assert_eq!(
            escape(&"<".repeat(128 * 8 - 1)).to_string(),
            "&lt;".repeat(128 * 8 - 1)
        );
        assert_eq!(escape(string_long).to_string(), string_long);
        assert_eq!(
            escape(&[string_long, "<"].join("")).to_string(),
            [string_long, "&lt;"].join("")
        );
        assert_eq!(
            escape(&["<", string_long].join("")).to_string(),
            ["&lt;", string_long].join("")
        );
        assert_eq!(
            escape(&escapes.repeat(1024)).to_string(),
            escaped.repeat(1024)
        );
        assert_eq!(
            escape(&[string_long, "<", string_long].join("")).to_string(),
            [string_long, "&lt;", string_long].join("")
        );
        assert_eq!(
            escape(&[string_long, "<", string_long, escapes, string_long,].join("")).to_string(),
            [string_long, "&lt;", string_long, escaped, string_long,].join("")
        );
    }
}
