// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub mod riscv;
