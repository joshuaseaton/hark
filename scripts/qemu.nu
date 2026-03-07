#!/usr/bin/env nu

# Copyright (c) 2026 Joshua Seaton
#
# Use of this source code is governed by a MIT-style
# license that can be found in the LICENSE file or at
# https://opensource.org/licenses/MIT

use build.nu

# If a QEMU board is set, this will build the example Hark app and run it
# under the appropriate QEMU configuration.
def main [
    --dry      # Return the QEMU command (as a list of strings) without running it
    --release  # Build in release mode
] {
    cd $env.FILE_PWD
    cd ..

    let result = if $release { build --release } else { build }
    let flattened = $result.flattened
    let qemu_settings = open $result.config | get qemu

    mut command = [ $"qemu-system-($qemu_settings.arch)" -nographic]
    $command = $command | append (match $qemu_settings.arch {
        "riscv32" | "riscv64" =>  [ -bios $flattened ]
        _ => [ -kernel $flattened ]
    })

    let board_flags = $qemu_settings
        | reject arch  # Used for picking out the QEMU binary itself.
        | transpose key value
        | each { |setting|
            let flag = match $setting.key {
                "cpu" => "-cpu",
                "machine" => "-machine"
                "memory" => "-m"
                _ => { error make --unspanned $"unknown QEMU setting \"($setting.key)\"" }
            }
            [$flag $setting.value]
        }
        | flatten

    $command = $command | append $board_flags

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
