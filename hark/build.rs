// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use std::env;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde::Deserialize;

#[derive(Deserialize)]
struct Spec {
    platform: String,
    boot_flash_address: u64,
    boot_flash_size: u64,
    boot_ram_address: u64,
    boot_ram_size: u64,
}

fn declare_input(path: impl Display) {
    println!("cargo::rerun-if-changed={path}");
}

#[allow(unused)]
fn set_cfg_name(key: &str, conditon: bool) {
    if conditon {
        println!("cargo::rustc-cfg={key}");
    }
    println!("cargo::rustc-check-cfg=cfg({key})");
}

fn set_cfg_pair(key: &str, value: impl Display, value_predicate: &str) {
    println!("cargo::rustc-cfg={key}=\"{value}\"");
    println!("cargo::rustc-check-cfg=cfg({key}, values({value_predicate}))");
}

fn set_env(key: &str, value: &str) {
    println!("cargo::rustc-env={key}={value}");
}

fn main() {
    let Ok(board_env) = env::var("HARK_BOARD") else {
        panic!(concat!(
            "$HARK_BOARD must be set either to the name of a supported board ",
            "or the absolute path to a custom TOML board spec"
        ));
    };

    let cwd = env::current_dir().unwrap();
    let board = PathBuf::from(&board_env);
    let board_toml = if board.is_absolute() {
        board
    } else {
        cwd.clone()
            .join("board")
            .join(board_env)
            .with_added_extension("toml")
    };
    assert!(
        board_toml.exists(),
        "$HARK_BOARD is invalid: {} does not exist",
        board_toml.display()
    );

    declare_input(board_toml.display());

    let board_toml_contents = fs::read_to_string(board_toml).unwrap();
    let spec: Spec = toml::from_str(&board_toml_contents).unwrap();

    set_cfg_pair("platform", spec.platform, "any()");

    let linker_script = cwd.join("src").join("kernel.ld");
    declare_input(linker_script.display());

    let link_args = [
        format!("-T{}", linker_script.display()),
        format!("--defsym=BOOT_FLASH_ADDRESS={:#x}", spec.boot_flash_address),
        format!("--defsym=BOOT_FLASH_SIZE={:#x}", spec.boot_flash_size),
        format!("--defsym=BOOT_RAM_ADDRESS={:#x}", spec.boot_ram_address),
        format!("--defsym=BOOT_RAM_SIZE={:#x}", spec.boot_ram_size),
        "--build-id".to_string(),
    ];
    hark_build::emit_metadata_for_app(link_args.as_slice());

    set_env("HARK_VERSION", env!("CARGO_PKG_VERSION"));

    let revision: String = Command::new("git")
        .args([
            "describe",
            "--always",
            "--dirty",
            "--abbrev=40",
            "--match=\"\"",
        ])
        .stdout(Stdio::piped())
        .output()
        .unwrap()
        .stdout
        .try_into()
        .unwrap();
    assert!(!revision.is_empty());
    set_env("HARK_REVISION", &revision);
}
