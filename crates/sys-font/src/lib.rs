//! The system font used by this CHIP8 emulator.

use strum::EnumCount;

/// The system font, with sprite data representing the hexadecimal numbers from
/// `0x0` thorugh `0xF`. All characters are 4 pixels wide by 5 pixels tall.
///
/// The enumeration itself can be used to access individual characters, while
/// the [`Font::get_table_as_bytes()`] method can be used to convert all
/// characters to a block of memory used to represent them by the CHIP-8, in the
/// correct order.
///
/// There existed at least four different CHIP-8 fonts in historical interpreters.
/// There was no real standard. So, this is just a font that I like.
#[derive(Debug, EnumCount, Copy, Clone)]
#[repr(u8)]
#[allow(dead_code)]
#[rustfmt::skip]
pub enum Font {
    Char0 = 0, Char1, Char2, Char3, Char4, Char5, Char6, Char7, Char8, Char9,
    CharA, CharB, CharC, CharD, CharE, CharF
}

impl Font {
    /// The preferred location in the system memory to load this font table
    /// to.
    ///
    /// It was popular for historical emulators to place the system font table
    /// from memory addresses `0x050`-`0x09F`, so `rust-chip` follows the same
    /// convention.
    pub const PREFERRED_TABLE_STARTING_ADDRESS: u16 = 0x050;

    /// Convert a character to the 5-byte sequence representing it in memory.
    #[allow(dead_code)]
    pub const fn as_bytes(&self) -> [u8; 5] {
        match self {
            Font::Char0 => [0xF0, 0x90, 0x90, 0x90, 0xF0],
            Font::Char1 => [0x20, 0x60, 0x20, 0x20, 0x70],
            Font::Char2 => [0xF0, 0x10, 0xF0, 0x80, 0xF0],
            Font::Char3 => [0xF0, 0x10, 0xF0, 0x10, 0xF0],
            Font::Char4 => [0x90, 0x90, 0xF0, 0x10, 0x10],
            Font::Char5 => [0xF0, 0x80, 0xF0, 0x10, 0xF0],
            Font::Char6 => [0xF0, 0x80, 0xF0, 0x90, 0xF0],
            Font::Char7 => [0xF0, 0x10, 0x20, 0x40, 0x40],
            Font::Char8 => [0xF0, 0x90, 0xF0, 0x90, 0xF0],
            Font::Char9 => [0xF0, 0x90, 0xF0, 0x10, 0xF0],
            Font::CharA => [0xF0, 0x90, 0xF0, 0x90, 0x90],
            Font::CharB => [0xE0, 0x90, 0xE0, 0x90, 0xE0],
            Font::CharC => [0xF0, 0x80, 0x80, 0x80, 0xF0],
            Font::CharD => [0xE0, 0x90, 0x90, 0x90, 0xE0],
            Font::CharE => [0xF0, 0x80, 0xF0, 0x80, 0xF0],
            Font::CharF => [0xF0, 0x80, 0xF0, 0x80, 0x80],
        }
    }

    /// Get all the characters in this font as a single contiguous hunk of memory,
    /// ready for loading into RAM.
    ///
    /// If possible, consider loading this at the memory location given by
    /// [`Font::PREFERRED_TABLE_STARTING_ADDRESS`].
    #[rustfmt::skip]
    pub const fn get_table_as_bytes() -> [u8; 80] {
        [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ]
    }

    /// Get the offset, in bytes, of a character into the font table returned
    /// by [`Font::get_table_as_bytes()`].
    #[allow(dead_code)]
    pub const fn table_offset(&self) -> usize {
        ((*self as u8) * 5) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_in_correct_order_and_bytes_are_correct() {
        use Font::*;

        const TABLE: [u8; 80] = Font::get_table_as_bytes();

        assert_eq!(
            &TABLE[Char0.table_offset()..Char1.table_offset()],
            &Char0.as_bytes()[..]
        );
        assert_eq!(
            &TABLE[Char1.table_offset()..Char2.table_offset()],
            &Char1.as_bytes()[..]
        );
        assert_eq!(
            &TABLE[Char2.table_offset()..Char3.table_offset()],
            &Char2.as_bytes()[..]
        );
        assert_eq!(
            &TABLE[Char3.table_offset()..Char4.table_offset()],
            &Char3.as_bytes()[..]
        );
        assert_eq!(
            &TABLE[Char4.table_offset()..Char5.table_offset()],
            &Char4.as_bytes()[..]
        );
        assert_eq!(
            &TABLE[Char5.table_offset()..Char6.table_offset()],
            &Char5.as_bytes()[..]
        );
        assert_eq!(
            &TABLE[Char6.table_offset()..Char7.table_offset()],
            &Char6.as_bytes()[..]
        );
        assert_eq!(
            &TABLE[Char7.table_offset()..Char8.table_offset()],
            &Char7.as_bytes()[..]
        );
        assert_eq!(
            &TABLE[Char8.table_offset()..Char9.table_offset()],
            &Char8.as_bytes()[..]
        );
        assert_eq!(
            &TABLE[Char9.table_offset()..CharA.table_offset()],
            &Char9.as_bytes()[..]
        );
        assert_eq!(
            &TABLE[CharA.table_offset()..CharB.table_offset()],
            &CharA.as_bytes()[..]
        );
        assert_eq!(
            &TABLE[CharB.table_offset()..CharC.table_offset()],
            &CharB.as_bytes()[..]
        );
        assert_eq!(
            &TABLE[CharC.table_offset()..CharD.table_offset()],
            &CharC.as_bytes()[..]
        );
        assert_eq!(
            &TABLE[CharD.table_offset()..CharE.table_offset()],
            &CharD.as_bytes()[..]
        );
        assert_eq!(
            &TABLE[CharE.table_offset()..CharF.table_offset()],
            &CharE.as_bytes()[..]
        );
        assert_eq!(
            &TABLE[CharF.table_offset()..Font::COUNT * 5],
            &CharF.as_bytes()[..]
        );
    }
}
