//! Minimal tty key reader that consumes exactly one key per call.
//!
//! Crossterm reads up to 1 KiB from the tty in a single `read(2)` and parses
//! everything it got. If the quit key and the first key of the next shell
//! command arrive in the same read, that key is consumed and lost when the
//! process exits. Reading one byte at a time and stopping as soon as a complete
//! key has been assembled leaves any following key in the kernel input queue,
//! where the shell will pick it up.

use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use signal_hook::consts::SIGWINCH;
use signal_hook::iterator::Signals;

use crate::event::Event;

/// How long to wait for the rest of an escape sequence after ESC. Terminals
/// write a key sequence in one go, so this only needs to cover a split write;
/// keeping it short avoids treating a fast next key as Alt+key.
const ESC_TIMEOUT: Duration = Duration::from_millis(5);
/// How long to wait for the remaining bytes of a sequence already started.
const SEQUENCE_TIMEOUT: Duration = Duration::from_millis(50);

#[derive(Debug)]
pub struct TtyInput {
    fd: RawFd,
    _tty: Option<File>,
    resize_fd: RawFd,
    pending: RefCell<Option<Event>>,
}

impl TtyInput {
    pub fn new() -> Result<Self> {
        let (fd, tty) = if unsafe { libc::isatty(libc::STDIN_FILENO) } == 1 {
            (libc::STDIN_FILENO, None)
        } else {
            let file = File::options()
                .read(true)
                .write(true)
                .open("/dev/tty")
                .context("opening /dev/tty")?;
            let fd = file.as_raw_fd();
            (fd, Some(file))
        };

        let mut pipe_fds = [0 as libc::c_int; 2];
        if unsafe { libc::pipe(pipe_fds.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error()).context("creating resize pipe");
        }
        let resize_fd = pipe_fds[0];
        let resize_write_fd = pipe_fds[1];

        let mut signals = Signals::new([SIGWINCH]).context("listening for SIGWINCH")?;
        thread::spawn(move || {
            for _ in signals.forever() {
                let byte = [1u8];
                unsafe {
                    libc::write(
                        resize_write_fd,
                        byte.as_ptr() as *const libc::c_void,
                        byte.len(),
                    );
                }
            }
        });

        Ok(Self {
            fd,
            _tty: tty,
            resize_fd,
            pending: RefCell::new(None),
        })
    }

    /// Wait up to `timeout` for one key or a resize. `None` means the timeout
    /// expired. Only the bytes of the returned key are consumed.
    pub fn next_event(&self, timeout: Duration) -> Result<Option<Event>> {
        if let Some(event) = self.pending.borrow_mut().take() {
            return Ok(Some(event));
        }

        let mut poll_fds = [
            libc::pollfd {
                fd: self.fd,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: self.resize_fd,
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        let millis = timeout.as_millis().min(i32::MAX as u128) as i32;
        let ready = unsafe { libc::poll(poll_fds.as_mut_ptr(), 2, millis) };
        if ready < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                return Ok(None);
            }
            return Err(error.into());
        }
        if ready == 0 {
            return Ok(None);
        }

        if poll_fds[1].revents & libc::POLLIN != 0 {
            self.drain_resize();
            return Ok(Some(Event::Resize));
        }
        if poll_fds[0].revents & libc::POLLIN != 0 {
            return self.read_key().map(|key| key.map(Event::Key));
        }
        if poll_fds[0].revents & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL) != 0 {
            return Err(anyhow::anyhow!("terminal input closed"));
        }
        Ok(None)
    }

    fn drain_resize(&self) {
        let mut buffer = [0u8; 64];
        unsafe {
            libc::read(
                self.resize_fd,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
            );
        }
    }

    fn read_key(&self) -> Result<Option<KeyEvent>> {
        let Some(first) = self.read_byte(None)? else {
            return Ok(None);
        };
        self.parse_key(first)
    }

    fn parse_key(&self, first: u8) -> Result<Option<KeyEvent>> {
        match first {
            0x1b => self.parse_escape(),
            b'\r' | b'\n' => Ok(Some(key(KeyCode::Enter, KeyModifiers::NONE))),
            b'\t' => Ok(Some(key(KeyCode::Tab, KeyModifiers::NONE))),
            0x7f | 0x08 => Ok(Some(key(KeyCode::Backspace, KeyModifiers::NONE))),
            0x00 => Ok(Some(key(
                KeyCode::Char(' '),
                KeyModifiers::CONTROL,
            ))),
            0x01..=0x1a => {
                let letter = (b'a' + first - 1) as char;
                Ok(Some(key(KeyCode::Char(letter), KeyModifiers::CONTROL)))
            }
            0x1c..=0x1f => Ok(None),
            b' '..=b'~' => Ok(Some(key(
                KeyCode::Char(first as char),
                KeyModifiers::NONE,
            ))),
            0x80..=0xff => {
                let Some(character) = self.read_utf8(first)? else {
                    return Ok(None);
                };
                Ok(Some(key(
                    KeyCode::Char(character),
                    KeyModifiers::NONE,
                )))
            }
        }
    }

    fn parse_escape(&self) -> Result<Option<KeyEvent>> {
        let Some(second) = self.read_byte(Some(ESC_TIMEOUT))? else {
            return Ok(Some(key(KeyCode::Esc, KeyModifiers::NONE)));
        };
        match second {
            b'[' => self.parse_csi(),
            b'O' => self.parse_ss3(),
            // Two Escs typed quickly: report the first now, keep the second for
            // the next call instead of swallowing both.
            0x1b => {
                let esc = key(KeyCode::Esc, KeyModifiers::NONE);
                *self.pending.borrow_mut() = Some(Event::Key(esc));
                Ok(Some(esc))
            }
            b'\r' => Ok(Some(key(KeyCode::Enter, KeyModifiers::ALT))),
            b'\t' => Ok(Some(key(KeyCode::Tab, KeyModifiers::ALT))),
            0x7f | 0x08 => Ok(Some(key(KeyCode::Backspace, KeyModifiers::ALT))),
            0x01..=0x1a => {
                let letter = (b'a' + second - 1) as char;
                Ok(Some(key(
                    KeyCode::Char(letter),
                    KeyModifiers::ALT | KeyModifiers::CONTROL,
                )))
            }
            b' '..=b'~' => Ok(Some(key(
                KeyCode::Char(second as char),
                KeyModifiers::ALT,
            ))),
            0x80..=0xff => {
                let Some(character) = self.read_utf8(second)? else {
                    return Ok(None);
                };
                Ok(Some(key(KeyCode::Char(character), KeyModifiers::ALT)))
            }
            _ => Ok(None),
        }
    }

    fn parse_csi(&self) -> Result<Option<KeyEvent>> {
        let mut parameters = String::new();
        let final_byte = loop {
            let Some(byte) = self.read_byte(Some(SEQUENCE_TIMEOUT))? else {
                return Ok(None);
            };
            if (0x40..=0x7e).contains(&byte) {
                break byte;
            }
            if byte.is_ascii_digit()
                || matches!(byte, b';' | b'<' | b'=' | b'>' | b'?')
            {
                parameters.push(byte as char);
                continue;
            }
            return Ok(None);
        };

        if parameters.starts_with('<') {
            // SGR mouse report; mouse reporting is not enabled by fpick.
            return Ok(None);
        }

        let parts: Vec<&str> = parameters.split(';').collect();
        let number = |index: usize| parts.get(index).and_then(|part| part.parse::<u32>().ok());
        let code = match final_byte {
            b'A' => Some(KeyCode::Up),
            b'B' => Some(KeyCode::Down),
            b'C' => Some(KeyCode::Right),
            b'D' => Some(KeyCode::Left),
            b'H' => Some(KeyCode::Home),
            b'F' => Some(KeyCode::End),
            b'Z' => Some(KeyCode::BackTab),
            b'~' => match number(0) {
                Some(1) | Some(7) => Some(KeyCode::Home),
                Some(2) => Some(KeyCode::Insert),
                Some(3) => Some(KeyCode::Delete),
                Some(4) | Some(8) => Some(KeyCode::End),
                Some(5) => Some(KeyCode::PageUp),
                Some(6) => Some(KeyCode::PageDown),
                Some(11) => Some(KeyCode::F(1)),
                Some(12) => Some(KeyCode::F(2)),
                Some(13) => Some(KeyCode::F(3)),
                Some(14) => Some(KeyCode::F(4)),
                Some(15) => Some(KeyCode::F(5)),
                Some(17) => Some(KeyCode::F(6)),
                Some(18) => Some(KeyCode::F(7)),
                Some(19) => Some(KeyCode::F(8)),
                Some(20) => Some(KeyCode::F(9)),
                Some(21) => Some(KeyCode::F(10)),
                Some(23) => Some(KeyCode::F(11)),
                Some(24) => Some(KeyCode::F(12)),
                _ => None,
            },
            _ => None,
        };
        Ok(code.map(|code| key(code, modifier_from(number(1)))))
    }

    fn parse_ss3(&self) -> Result<Option<KeyEvent>> {
        let Some(byte) = self.read_byte(Some(SEQUENCE_TIMEOUT))? else {
            return Ok(None);
        };
        let code = match byte {
            b'A' => KeyCode::Up,
            b'B' => KeyCode::Down,
            b'C' => KeyCode::Right,
            b'D' => KeyCode::Left,
            b'H' => KeyCode::Home,
            b'F' => KeyCode::End,
            b'P' => KeyCode::F(1),
            b'Q' => KeyCode::F(2),
            b'R' => KeyCode::F(3),
            b'S' => KeyCode::F(4),
            _ => return Ok(None),
        };
        Ok(Some(key(code, KeyModifiers::NONE)))
    }

    fn read_utf8(&self, first: u8) -> Result<Option<char>> {
        let length = match first {
            0xc2..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xf4 => 4,
            _ => return Ok(None),
        };
        let mut bytes = [0u8; 4];
        bytes[0] = first;
        for byte in bytes.iter_mut().take(length).skip(1) {
            let Some(value) = self.read_byte(Some(SEQUENCE_TIMEOUT))? else {
                return Ok(None);
            };
            *byte = value;
        }
        Ok(std::str::from_utf8(&bytes[..length])
            .ok()
            .and_then(|text| text.chars().next()))
    }

    /// Read exactly one byte. With a timeout, `None` means nothing arrived in
    /// time; otherwise the call blocks until a byte is available.
    fn read_byte(&self, timeout: Option<Duration>) -> io::Result<Option<u8>> {
        if let Some(timeout) = timeout {
            let mut poll_fd = libc::pollfd {
                fd: self.fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let millis = timeout.as_millis().min(i32::MAX as u128) as i32;
            let ready = unsafe { libc::poll(&mut poll_fd, 1, millis) };
            if ready < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    return Ok(None);
                }
                return Err(error);
            }
            if ready == 0 {
                return Ok(None);
            }
        }

        let mut byte = 0u8;
        let read = unsafe {
            libc::read(
                self.fd,
                &mut byte as *mut u8 as *mut libc::c_void,
                1,
            )
        };
        if read < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                return Ok(None);
            }
            return Err(error);
        }
        if read == 0 {
            return Ok(None);
        }
        Ok(Some(byte))
    }
}

fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

/// Decode the xterm modifier parameter: 1 means none, then Shift=1, Alt=2, Ctrl=4.
fn modifier_from(code: Option<u32>) -> KeyModifiers {
    let Some(code) = code else {
        return KeyModifiers::NONE;
    };
    let bits = code.saturating_sub(1);
    let mut modifiers = KeyModifiers::NONE;
    if bits & 1 != 0 {
        modifiers |= KeyModifiers::SHIFT;
    }
    if bits & 2 != 0 {
        modifiers |= KeyModifiers::ALT;
    }
    if bits & 4 != 0 {
        modifiers |= KeyModifiers::CONTROL;
    }
    modifiers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reader_with(bytes: &[u8]) -> TtyInput {
        let mut fds = [0 as libc::c_int; 2];
        assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
        let written = unsafe {
            libc::write(fds[1], bytes.as_ptr() as *const libc::c_void, bytes.len())
        };
        assert_eq!(written as usize, bytes.len());
        unsafe { libc::close(fds[1]) };
        TtyInput {
            fd: fds[0],
            _tty: None,
            resize_fd: -1,
            pending: RefCell::new(None),
        }
    }

    fn parse(bytes: &[u8]) -> Option<KeyEvent> {
        let reader = reader_with(bytes);
        let first = reader.read_byte(None).unwrap().unwrap();
        reader.parse_key(first).unwrap()
    }

    fn assert_key(bytes: &[u8], code: KeyCode, modifiers: KeyModifiers) {
        let event = parse(bytes).expect("no key parsed");
        assert_eq!(event.code, code, "code for {:?}", bytes);
        assert_eq!(event.modifiers, modifiers, "modifiers for {:?}", bytes);
    }

    #[test]
    fn parses_plain_and_control_characters() {
        assert_key(b"a", KeyCode::Char('a'), KeyModifiers::NONE);
        assert_key(b" ", KeyCode::Char(' '), KeyModifiers::NONE);
        assert_key(b"\x03", KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_key(b"\x15", KeyCode::Char('u'), KeyModifiers::CONTROL);
        assert_key(b"\x17", KeyCode::Char('w'), KeyModifiers::CONTROL);
        assert_key(b"\r", KeyCode::Enter, KeyModifiers::NONE);
        assert_key(b"\t", KeyCode::Tab, KeyModifiers::NONE);
        assert_key(b"\x7f", KeyCode::Backspace, KeyModifiers::NONE);
    }

    #[test]
    fn parses_arrows_and_editing_keys() {
        assert_key(b"\x1b[A", KeyCode::Up, KeyModifiers::NONE);
        assert_key(b"\x1b[B", KeyCode::Down, KeyModifiers::NONE);
        assert_key(b"\x1b[C", KeyCode::Right, KeyModifiers::NONE);
        assert_key(b"\x1b[D", KeyCode::Left, KeyModifiers::NONE);
        assert_key(b"\x1b[H", KeyCode::Home, KeyModifiers::NONE);
        assert_key(b"\x1b[F", KeyCode::End, KeyModifiers::NONE);
        assert_key(b"\x1b[3~", KeyCode::Delete, KeyModifiers::NONE);
        assert_key(b"\x1b[5~", KeyCode::PageUp, KeyModifiers::NONE);
        assert_key(b"\x1b[6~", KeyCode::PageDown, KeyModifiers::NONE);
        assert_key(b"\x1b[1;5C", KeyCode::Right, KeyModifiers::CONTROL);
        assert_key(b"\x1b[1;3D", KeyCode::Left, KeyModifiers::ALT);
    }

    #[test]
    fn parses_function_keys() {
        assert_key(b"\x1bOQ", KeyCode::F(2), KeyModifiers::NONE);
        assert_key(b"\x1bOS", KeyCode::F(4), KeyModifiers::NONE);
        assert_key(b"\x1b[15~", KeyCode::F(5), KeyModifiers::NONE);
    }

    #[test]
    fn parses_alt_combinations() {
        assert_key(b"\x1b\r", KeyCode::Enter, KeyModifiers::ALT);
        assert_key(b"\x1b\x7f", KeyCode::Backspace, KeyModifiers::ALT);
        assert_key(b"\x1bx", KeyCode::Char('x'), KeyModifiers::ALT);
    }

    #[test]
    fn parses_utf8_characters() {
        assert_key("ż".as_bytes(), KeyCode::Char('ż'), KeyModifiers::NONE);
        assert_key("→".as_bytes(), KeyCode::Char('→'), KeyModifiers::NONE);
    }

    #[test]
    fn double_escape_is_two_escape_keys() {
        let reader = reader_with(b"\x1b\x1b");
        let first = reader.read_key().unwrap().unwrap();
        assert_eq!(first.code, KeyCode::Esc);
        let second = reader.next_event(Duration::ZERO).unwrap().unwrap();
        match second {
            Event::Key(key_event) => assert_eq!(key_event.code, KeyCode::Esc),
            Event::Resize => panic!("expected the buffered Esc"),
        }
    }

    #[test]
    fn decodes_modifier_parameters() {
        assert_eq!(modifier_from(None), KeyModifiers::NONE);
        assert_eq!(modifier_from(Some(1)), KeyModifiers::NONE);
        assert_eq!(modifier_from(Some(2)), KeyModifiers::SHIFT);
        assert_eq!(modifier_from(Some(3)), KeyModifiers::ALT);
        assert_eq!(modifier_from(Some(5)), KeyModifiers::CONTROL);
        assert_eq!(
            modifier_from(Some(8)),
            KeyModifiers::SHIFT | KeyModifiers::ALT | KeyModifiers::CONTROL
        );
    }
}
