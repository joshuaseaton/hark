// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

pub mod sifive_test;
pub mod uart;

use core::fmt;

pub(crate) trait Console {
    fn describe(&self, w: &mut impl fmt::Write);
    fn write(&self, bytes: &[u8]);
}
