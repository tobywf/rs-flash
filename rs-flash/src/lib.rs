// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Dump,
    Load,
}

impl Direction {
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        match self {
            Self::Dump => 1,
            Self::Load => 2,
        }
    }

    #[inline]
    pub const fn from_u32(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::Dump),
            2 => Some(Self::Load),
            _ => None,
        }
    }
}

/// Sets up the flash interface.
///
/// This exports the necessary information, and provides `RS_FLASH_BUFFER` and
/// `RS_FLASH_CONTROL` for communicating with the host.
///
/// # Usage
///
/// For programs dumping flash, use:
/// ```
/// const FLASH_SIZE: usize = 16 * 1024 * 1024;
/// const BUFFER_SIZE: usize = 32 * 1024;
/// flash_interface!(FLASH_SIZE, BUFFER_SIZE, dump);
/// ```
///
/// For programs loading flash, use `load` instead of `dump`.
#[macro_export]
macro_rules! flash_interface {
    ($flash_size:ident, $buffer_size:ident, dump) => {
        flash_interface!(@ $flash_size, $buffer_size, $crate::Direction::Dump);
    };
    ($flash_size:ident, $buffer_size:ident, load) => {
        flash_interface!(@ $flash_size, $buffer_size, $crate::Direction::Load);
    };
    (@ $flash_size:ident, $buffer_size:ident, $direction:path) => {
        /// The number of chunks required to read or write the entire flash.
        const _CHUNKS: usize = $flash_size / $buffer_size;
        /// Assert that the flash size is a multiple of the buffer size.
        const fn _assert_buffer_size() {
            if $flash_size != $buffer_size * _CHUNKS {
                ::core::panic!("Invalid buffer size, must divide flash size without remainder");
            }
        }
        const _ASSERT_BUFFER_SIZE: () = _assert_buffer_size();

        #[link_section = ".rs-flash"]
        #[used]
        #[no_mangle]
        /// Exported flash information (for the host program).
        static _RS_FLASH_TABLE: [u32; 3] = [
            $flash_size as _,
            $buffer_size as _,
            $direction.as_u32(),
        ];

        #[export_name = "_RS_FLASH_BUFFER"]
        /// Buffer in RAM for dumping or loading flash contents.
        static mut RS_FLASH_BUFFER: ::core::mem::MaybeUninit<[u8; $buffer_size]> = ::core::mem::MaybeUninit::uninit();
        #[export_name = "_RS_FLASH_CONTROL"]
        /// Control signalling between the target and the host.
        static mut RS_FLASH_CONTROL: ::core::sync::atomic::AtomicUsize = ::core::sync::atomic::AtomicUsize::new(0);
    }
}
