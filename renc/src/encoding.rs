use redt::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
    Ascii,
    Utf8,
}

impl Encoding {
    #[inline]
    pub const fn name(&self) -> &'static str {
        match self {
            Encoding::Ascii => "ASCII",
            Encoding::Utf8 => "UTF-8",
        }
    }

    #[inline]
    pub const fn allows_surrogates(&self) -> bool {
        match self {
            Encoding::Ascii => false,
            Encoding::Utf8 => false,
        }
    }

    #[inline]
    pub const fn min_codepoint(&self) -> u32 {
        match self {
            Encoding::Ascii => 0,
            Encoding::Utf8 => 0,
        }
    }

    #[inline]
    pub const fn max_codepoint(&self) -> u32 {
        match self {
            Encoding::Ascii => 0x7F,
            Encoding::Utf8 => 0x10FFFF,
        }
    }

    #[inline]
    pub fn codepoint_ranges(&self) -> &'static [Range<u32>] {
        static ASCII_RANGES: &[Range<u32>] = &[Range::new_unchecked_const(0, 0x7F)];
        static UTF_RANGES: &[Range<u32>] = &[
            Range::new_unchecked_const(0, 0xD7FF),
            Range::new_unchecked_const(0xE000, 0x10FFFF),
        ];

        match self {
            Encoding::Ascii => ASCII_RANGES,
            Encoding::Utf8 => UTF_RANGES,
        }
    }
}
