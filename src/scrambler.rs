// Vendored from greetd (https://git.sr.ht/~kennylevinsen/greetd)
// Copyright (C) 2019-2024 Kenny Levinsen
// Licensed under GPL-3.0-or-later

use std::ffi::CString;
use zeroize::Zeroize;

/// Scrambling overwrites a buffers content with the default value. Useful to
/// avoid leaving behind a heap littered with old secrets.
pub trait Scrambler {
    fn scramble(&mut self);
}

impl<T: Zeroize> Scrambler for Vec<T> {
    fn scramble(&mut self) {
        self.zeroize();
    }
}

impl Scrambler for String {
    fn scramble(&mut self) {
        self.zeroize();
    }
}

impl Scrambler for CString {
    fn scramble(&mut self) {
        unsafe {
            let ptr = self.as_ptr();
            let mut offset = 0;

            while *ptr.offset(offset) != 0 {
                std::ptr::write_volatile(ptr.offset(offset).cast_mut(), 0);
                offset += 1;
            }
        }
    }
}
