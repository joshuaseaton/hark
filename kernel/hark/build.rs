// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

const KERNEL_LD: &str = "kernel.ld";
const PLATFORM_LD_INC: &str = "platform.ld.inc";

const DEFAULT_RISCV_PLATFORM: &str = "qemu-virt-riscv";

fn declare_input(path: &Path) {
    println!("cargo::rerun-if-changed={}", path.to_str().unwrap());
}

fn main() {
    let (arch, default_platform) = match env::var("TARGET").unwrap().as_str() {
        "riscv64-unknown-hark" | "riscv64imac-unknown-none-elf" => {
            ("riscv", DEFAULT_RISCV_PLATFORM)
        }
        unknown => panic!("Unsupported target: {unknown}"),
    };

    // Make "kernel.ld" (and its included platform .inc) available as a relative
    // path to any links.
    {
        // TODO: parameterize platform via cfg.
        let platform_dir: PathBuf = ["src", "platform", default_platform].iter().collect();

        let linker_script_template: PathBuf = ["src", "arch", arch, KERNEL_LD].iter().collect();
        declare_input(&linker_script_template);

        let linker_script_platform_inc = platform_dir.join(PLATFORM_LD_INC);
        declare_input(&linker_script_platform_inc);

        // Translate cfg-related environment variables to similarly spelled
        // #defines, allowing the linker scripts parameterization when run
        // through the C preprocessor.
        let cfg_defines = env::vars().filter_map(|(var, _)| {
            if var.starts_with("CARGO_FEATURE_") {
                Some(format!("-D{var}"))
            } else {
                None
            }
        });

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let preprocessed_linker_script = File::create(out_dir.join(KERNEL_LD)).unwrap();

        // System cc is fine; we're just using the preprocessor.
        Command::new("cc")
            .args([
                "--preprocess",
                "--no-line-commands",
                "--language=c",
                "-I",
                platform_dir.to_str().unwrap(),
                linker_script_template.to_str().unwrap(),
            ])
            .args(cfg_defines)
            .stdout(preprocessed_linker_script)
            .output()
            .expect("failed to preprocess linker script");

        println!("cargo::rustc-link-search={}", out_dir.display());
    }
}
