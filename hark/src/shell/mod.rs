// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use heapless::Vec;

use crate::platform::console;
use crate::print;
use crate::thread::Thread;

const BUFFER_SIZE: usize = 128;
const PROMPT: &[u8; 2] = b"$ ";

// Supported control sequences
#[derive(Clone, Copy, Debug)]
enum ControlSequence {
    // Valid prefixes
    Esc,      // Esc
    Csi,      // ESC [
    Csi1,     // ESC [ 1
    Csi1Sep,  // ESC [ 1 ;
    Csi3,     // ESC [ 3
    Csi1Sep5, // ESC [ 1 ; 5
    CsiSep,   // ESC [ ;
    CsiSep5,  // ESC [ ; 5

    // Full sequences
    Home,        // "ESC [ H"
    End,         // "ESC [ F"
    CursorLeft,  // "ESC [ D" or "ESC [ 1 D"
    CursorRight, // "ESC [ C" or "ESC [ 1 C"
    CursorUp,    // "ESC [ A"
    CursorDown,  // "ESC [ B"
    CtrlLeft,    // "ESC [ 1 ; 5 D" or "ESC [ ; 5 D"
    CtrlRight,   // "ESC [ 1 ; 5 C" or "ESC [ ; 5 C"
    Delete,      // "ESC [ 3 ~"
}

impl ControlSequence {
    // Encodes the state machine for building up a supported control sequence
    // or a prefix of one.
    fn next(self, ch: u8) -> Option<Self> {
        match self {
            Self::Esc => (ch == b'[').then_some(Self::Csi),
            Self::Csi => match ch {
                b'1' => Some(Self::Csi1),
                b'3' => Some(Self::Csi3),
                b';' => Some(Self::CsiSep),
                b'H' => Some(Self::Home),
                b'F' => Some(Self::End),
                b'A' => Some(Self::CursorUp),
                b'B' => Some(Self::CursorDown),
                b'D' => Some(Self::CursorLeft),
                b'C' => Some(Self::CursorRight),
                _ => None,
            },
            Self::Csi1 => match ch {
                b';' => Some(Self::Csi1Sep),
                b'D' => Some(Self::CursorLeft),
                b'C' => Some(Self::CursorRight),
                _ => None,
            },
            Self::Csi1Sep => (ch == b'5').then_some(Self::Csi1Sep5),
            Self::Csi1Sep5 | Self::CsiSep5 => match ch {
                b'D' => Some(Self::CtrlLeft),
                b'C' => Some(Self::CtrlRight),
                _ => None,
            },
            Self::Csi3 => (ch == b'~').then_some(Self::Delete),
            Self::CsiSep => (ch == b'5').then_some(Self::CsiSep5),
            _ => None,
        }
    }
}

// Control characters
const CTRL_C: u8 = 0x03;
const CURSOR_LEFT: u8 = 0x08; // Easier to emit for one-offs than the sequence.
const ENTER: u8 = 0x0d;
const ESC: u8 = 0x1b;
const BACKSPACE: u8 = 0x7f;

pub(super) fn run_in_background() {
    let thread = Thread::with_stack_size(|| enter(), 0x1000);
    thread.start();
}

fn enter() -> ! {
    console::write(b"Entering shell...\n\r");
    console::write(PROMPT);

    let mut displayed: Vec<u8, BUFFER_SIZE> = Vec::new();
    let mut cursor = 0;

    let mut buffer = [0u8; BUFFER_SIZE];
    let mut ctrl_seq: Option<ControlSequence> = None;
    loop {
        let num_buffered = console::read(&mut buffer);
        if num_buffered == 0 {
            continue;
        }

        for ch in buffer[..num_buffered].iter().copied() {
            // See if we're in the midst of building up a control sequence.
            if let Some(seq) = ctrl_seq {
                ctrl_seq = seq.next(ch);
                if let Some(seq) = ctrl_seq {
                    if handle_control_sequence(&mut displayed, &mut cursor, seq) {
                        // Success! reset the sequence.
                        ctrl_seq = None;
                    }
                    // Either we handled a control sequence or are still in the
                    // midst of building one up. In either case, continue;
                    continue;
                }
                // `ch` isn't the next character in a supported control
                // sequence: the sequence is dropped and we fall through to normal
                // character handling.
            }

            match ch {
                // We clear the whole line.
                CTRL_C => {
                    console::write_byte(b'\r');
                    for _ in 0..(PROMPT.len() + displayed.len()) {
                        console::write_byte(b' ');
                    }
                    console::write_byte(b'\r');
                    console::write(PROMPT);
                    cursor = 0; // Reset by \r
                    displayed.clear();
                }
                ENTER => {
                    console::write(b"\n\r");
                    console::write(PROMPT);
                    cursor = 0; // Reset by \r
                    displayed.clear();
                }
                ESC => ctrl_seq = Some(ControlSequence::Esc),

                0x20..=0x7e => {
                    if cursor == displayed.len() {
                        displayed.push(ch).unwrap();
                    } else {
                        displayed.insert(cursor, ch).unwrap();
                    }
                    let new_cursor = cursor + 1;
                    console::write(&displayed[cursor..]);
                    cursor = displayed.len();
                    move_cursor(&mut cursor, new_cursor);
                }
                BACKSPACE => {
                    if cursor > 0 {
                        displayed.remove(cursor - 1);
                        let new_cursor = cursor - 1;
                        console::write_byte(CURSOR_LEFT);
                        console::write(&displayed[cursor - 1..]);
                        console::write_byte(b' ');
                        cursor = displayed.len() + 1;
                        move_cursor(&mut cursor, new_cursor);
                    }
                }
                _ => {}
            }
        }
        crate::thread::yield_now();
    }
}

fn handle_control_sequence(
    displayed: &mut Vec<u8, BUFFER_SIZE>,
    cursor: &mut usize,
    seq: ControlSequence,
) -> bool {
    match seq {
        ControlSequence::Home => {
            move_cursor(cursor, 0);
        }
        ControlSequence::End => {
            move_cursor(cursor, displayed.len());
        }
        ControlSequence::CtrlLeft => {
            if *cursor == 0 {
                return true;
            }
            let mut idx = *cursor - 1;

            // Point back into the previous word.
            while idx > 0 && displayed[idx] == b' ' {
                idx -= 1;
            }

            // Point back to its beginning.
            while idx > 0 && displayed[idx - 1] != b' ' {
                idx -= 1;
            }
            move_cursor(cursor, idx);
        }
        ControlSequence::CtrlRight => {
            let mut idx = *cursor;

            // Point to the next whitespace
            while idx < displayed.len() && displayed[idx] != b' ' {
                idx += 1;
            }

            // Then point just past it.
            while idx < displayed.len() && displayed[idx] == b' ' {
                idx += 1;
            }
            move_cursor(cursor, idx);
        }
        ControlSequence::CursorLeft => {
            if *cursor > 0 {
                move_cursor(cursor, *cursor - 1);
            }
        }
        ControlSequence::CursorRight => {
            if *cursor < displayed.len() {
                move_cursor(cursor, *cursor + 1);
            }
        }
        ControlSequence::CursorUp | ControlSequence::CursorDown => {
            // TODO: history!
        }
        ControlSequence::Delete => {
            if *cursor < displayed.len() {
                displayed.remove(*cursor);
                let initial_cursor = *cursor;
                console::write(&displayed[*cursor..]);
                console::write_byte(b' ');
                *cursor = displayed.len() + 1;
                move_cursor(cursor, initial_cursor);
            }
        }
        _ => return false,
    }
    true
}

fn move_cursor(cursor: &mut usize, pos: usize) {
    if pos == *cursor {
        return;
    }
    console::write(b"\x1b[");
    if pos < *cursor {
        print!("{}D", *cursor - pos);
    } else {
        print!("{}C", pos - *cursor);
    }
    *cursor = pos;
}
