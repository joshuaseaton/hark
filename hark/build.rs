// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use serde::Deserialize;
use std::env;
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Spec {
    platform: String,
    arch: Arch,
    load_address: i64,
    options: Options,
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
    entry_mode: RiscvEntryMode,
}

#[derive(Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
enum RiscvEntryMode {
    M,
    S,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Options {
    riscv_sbi_console: bool,
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

fn set_env(key: &str, value: &str) {
    println!("cargo::rustc-env={key}={value}");
}

#[derive(Deserialize)]
struct CargoMetadata {
    target_directory: String,
}

fn final_artifact_dir() -> PathBuf {
    let target = env::var("TARGET").unwrap();
    let profile = env::var("PROFILE").unwrap();
    let metadata_stdout = Command::new("cargo")
        .arg("metadata")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .stdout
        .unwrap();
    let metadata: CargoMetadata = serde_json::from_reader(metadata_stdout).unwrap();
    PathBuf::from(metadata.target_directory)
        .join(target)
        .join(profile)
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

    // We write out the evaluated board specification to the final artifact
    // directory for easy inspection and programmatic access later on.
    let spec_json = final_artifact_dir().join("spec.json");
    let spec_json_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(&spec_json)
        .unwrap();
    Command::new("pkl")
        .args(pkl_args)
        .stdout(Stdio::from(spec_json_file))
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    // Now we read it back in to parameterize the build.
    let spec_json_file = File::open(spec_json).unwrap();
    let spec: Spec = serde_json::from_reader(spec_json_file).unwrap();

    set_cfg_pair("platform", spec.platform, "any()");
    match &spec.arch {
        Arch::Riscv(riscv) => {
            let m_mode = riscv.entry_mode == RiscvEntryMode::M;
            set_cfg_name("riscv_m_mode", m_mode);
            assert!(
                !(spec.options.riscv_sbi_console && m_mode),
                "No SBI in machine mode, so can't use it for a console"
            );
            set_cfg_name("riscv_sbi_console", spec.options.riscv_sbi_console);
        }
    }

    let linker_script = env::current_dir().unwrap().join("src").join("kernel.ld");
    declare_input(linker_script.display());

    let link_args = [
        format!("-T{}", linker_script.display()),
        format!("--defsym=LOAD_ADDRESS={:#x}", spec.load_address),
        "--build-id".to_string(),
    ];
    hark_build::emit_metadata_for_system(&link_args);

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
