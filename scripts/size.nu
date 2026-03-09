#!/usr/bin/env nu

# Copyright (c) 2026 Joshua Seaton
#
# Use of this source code is governed by a MIT-style
# license that can be found in the LICENSE file or at
# https://opensource.org/licenses/MIT

use build.nu

# Generate a summary of relevant sizes for the example app (e.g., by section)
export def main [
    --release  # Use a release build
] {
    let elf = if $release { build --release } else { build } | get elf
    let metadata = (
        ^llvm-readelf --elf-output-style=JSON --program-headers --sections $elf
            | from json
            | get 0
    )

    let totals = (
        $metadata.ProgramHeaders
            # PT_LOAD is 0b1
            | where $it.ProgramHeader.Type.Value == 0b1
            | reduce --fold { file: 0, writable: 0 } {
                |phdr, acc|
                let phdr = $phdr.ProgramHeader
                # PF_W is 0b10
                let writable_size = if (($phdr.Flags.Value | bits and 0x2) != 0) {
                    $phdr.MemSize
                } else {
                    0
                }

                {
                    file: ($acc.file + $phdr.FileSize),
                    writable: ($acc.writable + $writable_size),
                }
            }
    )

    let section_totals = (
        $metadata.Sections
            # SHF_ALLOC is 0b10
            | where ($it.Section.Flags.Value | bits and 0b10) != 0
            | each {
                |sec|
                let sec = $sec.Section
                {
                    name: $sec.Name.Name,
                    size: ($sec.Size | into filesize),
                }
            }
            | transpose --header-row
            | into record
    )

    {
        "Flash": ($totals.file | into filesize)
        "RAM": ($totals.writable | into filesize)
    } | merge $section_totals

}
