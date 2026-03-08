#!/usr/bin/env nu

# Copyright (c) 2026 Joshua Seaton
#
# Use of this source code is governed by a MIT-style
# license that can be found in the LICENSE file or at
# https://opensource.org/licenses/MIT

use build.nu

# If a QEMU board is set, this will build the example Hark app and run it
# under the appropriate QEMU configuration.
export def main [
    --dry      # Return the QEMU command (as a list of strings) without running it
    --release  # Build in release mode
] {
    const REPO_ROOT = path self | path dirname | path dirname
    cd $REPO_ROOT

    let result = if $release { build --release } else { build }
    let flattened = $result.flattened
    let qemu_settings = open $result.config | get qemu

    let board_flags = $qemu_settings
        | reject arch  # Used for picking out the QEMU binary itself.
        | transpose key value
        | each { |setting|
            let flag = match $setting.key {
                "cpu" => "-cpu",
                "machine" => "-machine"
                "memory" => "-m"
                "flash_size" => {
                    ^truncate -s $setting.value $flattened
                }
                _ => { error make --unspanned $"unknown QEMU setting \"($setting.key)\"" }
            }
            if ($flag | is-not-empty) {
                [$flag $setting.value]
            }
        }
        | flatten

    let arch = ^llvm-readelf --elf-output-style=JSON $result.elf
        | from json
        | get FileSummary.0.Arch

    let command = [
        $"qemu-system-($arch)"
         -bios none
        # Boot in place out of flash.
        -drive $"if=pflash,file=($flattened),format=raw,unit=0"
        -nographic
    ] | append $board_flags

    if $dry {
        return $command
    }

    # Set up .build-id directory for llvm-symbolizer --filter-markup.
    # TODO: Update llvm-symbolizer so that `llvm-symbolizer --obj $system` can
    # work without a .build-id directory. 
    let build_id = (^llvm-readelf -n $result.elf
        | lines
        | find "Build ID"
        | get 0
        | ansi strip
        | split row ": "
        | last
        | str trim)
    let bid_prefix = ($build_id | str substring 0..<2)
    let bid_suffix = ($build_id | str substring 2..)
    let bid_dir = [ target .build-id $bid_prefix ] | path join
    mkdir $bid_dir
    ^ln -sf ($result.elf| path expand) ([$bid_dir $"($bid_suffix).debug"] | path join)

    run-external $command.0 ...($command | skip 1)
        | (
            ^llvm-symbolizer
            --filter-markup
            --relativenames
            --color=never
            --debug-file-directory target/
        )
}
