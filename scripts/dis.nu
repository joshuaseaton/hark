#!/usr/bin/env nu

# Copyright (c) 2026 Joshua Seaton
#
# Use of this source code is governed by a MIT-style
# license that can be found in the LICENSE file or at
# https://opensource.org/licenses/MIT

use build.nu

# Disassembles the example hark app, writing the listing to a file next to
# the binary.
export def main [
    --release  # Disassemble the release build
] {
    const REPO_ROOT = path self | path dirname | path dirname
    cd $REPO_ROOT

    let app = if $release { build --release } else { build } | get elf
    let disasm = $"($app).lst"
    ^llvm-objdump --disassemble --demangle --line-numbers --no-show-raw-insn $app
        | save --force $disasm

    print $"Wrote: ($disasm)"

    # $EDITOR may contain other flags like `--wait` supplied in service of
    # `git commit` in one's terminal - but the base editor executable
    # followed by a file should always simply open the file for common
    # editors.
    if "EDITOR" in $env {
        let editor = $env.EDITOR | split row " " | first
        ^$editor $disasm
    }
}
