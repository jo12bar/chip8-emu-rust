use std::ops::{Bound, Index, IndexMut, Range, RangeBounds};

use crate::sys_font::Font;

/// The RAM is 4 kiB (4096 bytes) in size.
pub const RAM_SIZE: u16 = 4096;

/// The main system memory for a CHIP-8.
///
/// This memory is 4 kiB (4 kibibytes, or 4096 bytes) large. Since the CHIP8's
/// index register and program counter can only address 12 bits, which works
/// out to 4096 addresses, this is the perfect size.
///
/// All system memory is RAM, and all memory is writable. Program memory
/// is in the same overal memory pool as code. This allows for self-modifying
/// code.
///
/// The first CHIP-8 interpreter on the COSMAC VIP computer was located in the
/// same address space as the CHIP-8 code that it ran. The interpreter took
/// up addresses `0x000` to `0x1FF`, and it expected CHIP-8 programs to be
/// loaded into memory after it starting at address `0x200`. For compatibility,
/// `rust-chip` will *also* reserve the first 512 bytes (addresses `0x000`-`0x1FF`)
/// of memory for itself - this will be used for things like the system font.
/// `rust-chip` will not _prevent_ accesses to those parts of memory, but it
/// will load the program starting at address `0x1FF` and hope that the program
/// doesn't screw with system memory.
///
/// It's the wild west out there.
#[derive(Debug)]
pub struct Ram {
    mem: [u8; RAM_SIZE as usize],
}

impl Ram {
    pub fn new() -> Self {
        let mut mem = [0; RAM_SIZE as usize];

        /// Load the system font
        const FONT_TABLE: [u8; 80] = Font::get_table_as_bytes();
        for (i, byte) in FONT_TABLE.iter().enumerate() {
            mem[addr_to_usize(Font::PREFERRED_TABLE_STARTING_ADDRESS) + i] = *byte;
        }

        Self { mem }
    }

    /// Get an immutable reference to a single byte of memory at some address offset.
    ///
    /// Only the bottom 12 bits of `addr` are used for addressing.
    pub fn get(&self, addr: u16) -> &u8 {
        &self.mem[addr_to_usize(addr)]
    }

    /// Get a mutable reference to single byte of memory at some address offset.
    ///
    /// Only the bottom 12 bits of `addr` are used for addressing.
    pub fn get_mut(&mut self, addr: u16) -> &mut u8 {
        &mut self.mem[addr_to_usize(addr)]
    }

    /// Get an immutable reference to a range of memory at some address offset.
    ///
    /// Only the bottom 12 bits of each address in `addrs` are used for addressing.
    pub fn get_range<R>(&self, addr_range: R) -> &[u8]
    where
        R: RangeBounds<u16>,
    {
        &self.mem[addr_range_to_usize_range(addr_range)]
    }

    /// Get a mutable reference to a range of memory at some address offset.
    ///
    /// Only the bottom 12 bits of each address in `addrs` are used for addressing.
    pub fn get_range_mut<R>(&mut self, addr_range: R) -> &mut [u8]
    where
        R: RangeBounds<u16>,
    {
        &mut self.mem[addr_range_to_usize_range(addr_range)]
    }

    /// Set a single byte of memory at some address offset.
    ///
    /// Only the bottom 12 bits of `addr` are used for addressing.
    pub fn set(&mut self, addr: u16, val: u8) {
        self.mem[addr_to_usize(addr)] = val;
    }
}

impl Default for Ram {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<u16> for Ram {
    type Output = u8;

    fn index(&self, addr: u16) -> &Self::Output {
        self.get(addr)
    }
}

impl IndexMut<u16> for Ram {
    fn index_mut(&mut self, addr: u16) -> &mut Self::Output {
        self.get_mut(addr)
    }
}

/// Chop off the top 4 bits of a memory address, as all addresses are supposed
/// to be 12 bits in size.
#[inline]
const fn clean_addr(addr: u16) -> u16 {
    addr & 0b0000_1111_1111_1111
}

/// Little utility function to convert a memory address to a `usize` while
/// erasing the top 4 bits of the input `u16`.
#[inline]
const fn addr_to_usize(addr: u16) -> usize {
    clean_addr(addr) as usize
}

/// Normalizes an address range from all of the many `Range*` variants to just
/// a concrete `Range<u16>`, with the top 4 bits truncated.
fn normalize_addr_range<R>(addr_range: R) -> Range<u16>
where
    R: RangeBounds<u16>,
{
    let start_bound = addr_range.start_bound();
    let end_bound = addr_range.end_bound();

    let start = match start_bound {
        Bound::Included(addr) => clean_addr(*addr),
        Bound::Excluded(addr) => (clean_addr(*addr) + 1).min(RAM_SIZE - 1),
        Bound::Unbounded => 0,
    };

    let end = match end_bound {
        Bound::Included(addr) => (clean_addr(*addr) + 1).min(RAM_SIZE),
        Bound::Excluded(addr) => clean_addr(*addr),
        Bound::Unbounded => RAM_SIZE,
    };

    start..end
}

/// Converts an address range to a usize range, after normalizing it and
/// scrubbing the top 4 bits of all addresses.
#[inline]
fn addr_range_to_usize_range<R>(addr_range: R) -> Range<usize>
where
    R: RangeBounds<u16>,
{
    let normalized = normalize_addr_range(addr_range);

    (normalized.start as usize)..(normalized.end as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_font_loaded_correctly() {
        use Font::*;

        let ram = Ram::default();

        // utility function to convert a character range to a memory range
        let mem_range = |start: Font, end: Font| {
            ram.get_range(
                (start.table_offset() as u16 + Font::PREFERRED_TABLE_STARTING_ADDRESS)
                    ..(end.table_offset() as u16 + Font::PREFERRED_TABLE_STARTING_ADDRESS),
            )
        };

        assert_eq!(mem_range(Char0, Char1), &Char0.as_bytes()[..]);
        assert_eq!(mem_range(Char1, Char2), &Char1.as_bytes()[..]);
        assert_eq!(mem_range(Char2, Char3), &Char2.as_bytes()[..]);
        assert_eq!(mem_range(Char3, Char4), &Char3.as_bytes()[..]);
        assert_eq!(mem_range(Char4, Char5), &Char4.as_bytes()[..]);
        assert_eq!(mem_range(Char5, Char6), &Char5.as_bytes()[..]);
        assert_eq!(mem_range(Char6, Char7), &Char6.as_bytes()[..]);
        assert_eq!(mem_range(Char7, Char8), &Char7.as_bytes()[..]);
        assert_eq!(mem_range(Char8, Char9), &Char8.as_bytes()[..]);
        assert_eq!(mem_range(Char9, CharA), &Char9.as_bytes()[..]);
        assert_eq!(mem_range(CharA, CharB), &CharA.as_bytes()[..]);
        assert_eq!(mem_range(CharB, CharC), &CharB.as_bytes()[..]);
        assert_eq!(mem_range(CharC, CharD), &CharC.as_bytes()[..]);
        assert_eq!(mem_range(CharD, CharE), &CharD.as_bytes()[..]);
        assert_eq!(mem_range(CharE, CharF), &CharE.as_bytes()[..]);

        assert_eq!(
            ram.get_range(
                (CharF.table_offset() as u16 + Font::PREFERRED_TABLE_STARTING_ADDRESS)
                    ..=(CharF.table_offset() as u16 + Font::PREFERRED_TABLE_STARTING_ADDRESS + 4)
            ),
            &CharF.as_bytes()[..]
        );
    }
}
