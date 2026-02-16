// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

cfg_if::cfg_if! {
   if  #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        pub mod riscv;
        use riscv::Arch;
   }
}

trait ArchCommon {
    fn print_machine_context();
}

// Prints generic machine context, as enumerated from the CPU.
#[inline(never)]
pub(crate) fn print_machine_context() {
    Arch::print_machine_context();
}
