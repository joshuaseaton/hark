#!/usr/bin/env nu

# Copyright (c) 2026 Joshua Seaton
#
# Use of this source code is governed by a MIT-style
# license that can be found in the LICENSE file or at
# https://opensource.org/licenses/MIT

# Build and prepare the example app and return metadata
export def main [
    --release  # Build in release mode
] {
    cd $env.FILE_PWD
    cd ..

    let cargo_board_toml = [ .cargo board.toml ] | path join
    if not ($cargo_board_toml | path exists) {
        error make --unspanned "Run `scripts/set-board` first"
    }
    let board = open $cargo_board_toml | get env.HARK_BOARD

    let release_flag = if $release { [--release] } else { [] }
    let profile = if $release { "release" } else { "debug" }
    ^cargo build ...$release_flag

    let elf = [target $board $profile example] | path join
    let flattened = $"($elf).bin"
    ^llvm-objcopy -O binary $elf $flattened

    let board_toml = [ hark board $"($board).toml" ] | path join
    {
        config: $board_toml,
        elf: $elf,
        flattened: $flattened,
        
    }
}
