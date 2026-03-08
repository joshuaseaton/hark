#!/usr/bin/env nu

# Copyright (c) 2026 Joshua Seaton
#
# Use of this source code is governed by a MIT-style
# license that can be found in the LICENSE file or at
# https://opensource.org/licenses/MIT

# Crate names are lowercase identifiers; `[Unknown]` is a special case.
const CRATE_RE = '[a-z\[][\w\[\]]*'

# Analyze function bloat in the example app.
export def main [
    -n: int = 20   # Number of functions to show (0 for all)
    --release      # Build in release mode
    --crates       # Per-crate bloatedness
] {
    let release_flag = if $release { [--release] } else { [] }
    let crates_flag = if $crates { [--crates] } else { [] }
    # For some reason
    let lines = ^cargo bloat -w -n $n ...$release_flag ...$crates_flag | lines
    if $crates { $lines | by-crate } else { $lines | by-function }
}

# Per-crate bloatedness.
def by-crate []: list<string> -> table {
    let re = '^\s*(?<file>\S+)\s+(?<text>\S+)\s+(?<size>\S+)\s+(?<crate>' + $CRATE_RE + ')\s*$'
    $in
        | parse-table $re
        | select crate size text file
        | rename --column { text: ".text" }
}

# Per-function bloatedness.
def by-function []: list<string> -> table {
    let re = '^\s*(?<file>\S+)\s+(?<text>\S+)\s+(?<size>\S+)\s+(?<crate>' + $CRATE_RE + ')\s+(?<name>.+?)\s*$'
    $in
        | parse-table $re
        | select name size text file
        | rename --column { text: ".text" }
}

def parse-table [re: string]: list<string> -> table {
    skip until { |line| ($line | str trim) starts-with "File" }
        | skip 1
        | each { |line|
            let parsed = $line | parse -r $re
            if ($parsed | is-empty) { null } else { $parsed | first }
        }
        | compact
}

