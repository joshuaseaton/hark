// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use serde::Deserialize;
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Spec {
    platform: String,
    arch: Arch,
    load_address: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "name")]
enum Arch {
    #[serde(rename = "riscv")]
    Riscv(ArchRiscv),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArchRiscv {
    m_mode: bool,
}

fn declare_input(path: impl Display) {
    println!("cargo::rerun-if-changed={path}");
}

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

fn main() {
    let board_dir = PathBuf::from("board");
    let board_pkl = if let Ok(board) = env::var("HARK_BOARD") {
        board_dir.join(format!("{board}.pkl"))
    } else {
        PathBuf::from(
            env::var("HARK_BOARD_PKL").expect("neither $HARK_BOARD nor $HARK_BOARD_PKL wer set!"),
        )
    };
    let spec_pkl = board_dir.join("spec.pkl");
    declare_input(board_pkl.display());
    declare_input(spec_pkl.display());

    let mut pkl_args = vec![
        "eval".to_string(),
        "--format".to_string(),
        "json".to_string(),
        board_pkl.to_str().unwrap().to_string(),
    ];
    if let Ok(options) = env::var("HARK_OPTIONS")
        && Path::new(&options).exists()
    {
        declare_input(&options);
        pkl_args.push("--property".to_string());
        pkl_args.push(format!("options={options}"));
    }

    let pkl_eval_stdout = Command::new("pkl")
        .args(pkl_args)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .stdout
        .unwrap();
    let spec: Spec = serde_json::from_reader(pkl_eval_stdout).unwrap();

    //
    set_cfg_pair("platform", spec.platform.replace('-', "_"), "any()");
    match &spec.arch {
        Arch::Riscv(riscv) => {
            set_cfg_name("riscv_m_mode", riscv.m_mode);
        }
    }

    let linker_script_template = PathBuf::from("src").join("kernel.ld");
    declare_input(linker_script_template.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let linker_script = out_dir.join("kernel.ld");
    Command::new("cc") // System cc is fine for the preprocessor.
        .args([
            "--preprocess",
            "--no-line-commands",
            "--language=c",
            "-nostdinc", // Don't search the standard include paths
            "-undef",    // Undefine all predefined macros
            "-D",
            format!("LOAD_ADDRESS={:#x}", spec.load_address).as_str(),
            linker_script_template.to_str().unwrap(),
        ])
        .stdout(File::create(linker_script).unwrap())
        .output()
        .expect("failed to preprocess linker script");

    println!("cargo::rustc-link-search={}", out_dir.display());
}
