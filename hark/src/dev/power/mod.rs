// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

mod sifive_test;

pub use sifive_test::SiFiveTest;

pub trait Manager {
    fn shutdown() -> !;
    fn halt() -> !;
    fn reboot() -> !;
}
