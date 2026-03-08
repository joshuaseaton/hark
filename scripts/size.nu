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
            | reduce --fold { total: 0, writable_total: 0 } {
                |phdr, acc|
                let phdr = $phdr.ProgramHeader
                # PF_W is 0b10
                let writable_size = if ($phdr.Flags.Value | bits and 0x2) == 0 {
                    0
                } else {
                    $phdr.MemSize
                }

                {
                    total: ($acc.total + $phdr.MemSize),
                    writable_total: ($acc.writable_total + $writable_size),
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
        "Flash": ($totals.total | into filesize)
        "RAM": ($totals.writable_total | into filesize)
    } | merge $section_totals

}
