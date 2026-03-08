#!/usr/bin/env nu

# Copyright (c) 2026 Joshua Seaton
#
# Use of this source code is governed by a MIT-style
# license that can be found in the LICENSE file or at
# https://opensource.org/licenses/MIT

# Analyze function bloat in the example app.
export def main [
    --release  # Build in release mode
] {
    let release_flag = if $release { [--release] } else { [] }
    ^cargo bloat --message-format json ...$release_flag
        | from json
        | get functions
        | select name size crate  # Rearrange for readability
        | update size { into filesize }
}
