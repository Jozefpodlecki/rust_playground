use core::{fmt, slice};

use winapi::shared::ntdef::UNICODE_STRING;

pub struct CommandLineArgs(pub UNICODE_STRING);

impl CommandLineArgs {
    pub fn iter(&self) -> CommandLineIter<'_> {
        CommandLineIter::new(&self.0)
    }
}

impl<'a> IntoIterator for &'a CommandLineArgs {
    type Item = CommandLineArg<'a>;
    type IntoIter = CommandLineIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct CommandLineArg<'a>(&'a [u16]);

impl fmt::Display for CommandLineArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, arg) in self.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            
            let arg_str = arg.to_string::<260>();
            let arg_str = arg_str.as_str();
            
            if arg_str.contains(' ') || arg_str.contains('\t') {
                write!(f, "\"{}\"", arg_str)?;
            } else {
                write!(f, "{}", arg_str)?;
            }
        }
        Ok(())
    }
}

impl<'a> CommandLineArg<'a> {
    pub fn new(data: &'a [u16]) -> Self {
        Self(data)
    }

    pub fn as_u16_slice(&self) -> &'a [u16] {
        self.0
    }
    
    pub fn to_string<const N: usize>(&self) -> heapless::String<N> {
        let mut s = heapless::String::new();
        for c in char::decode_utf16(self.0.iter().cloned()) {
            if let Ok(c) = c {
                let _ = s.push(c);
            }
        }
        s
    }
}

impl<'a> AsRef<[u16]> for CommandLineArg<'a> {
    fn as_ref(&self) -> &[u16] {
        self.0
    }
}

impl<'a> fmt::Display for CommandLineArg<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in char::decode_utf16(self.0.iter().cloned()) {
            if let Ok(c) = c {
                write!(f, "{}", c)?;
            }
        }
        Ok(())
    }
}

pub struct CommandLineIter<'a> {
    data: &'a [u16],
    pos: usize,
    in_quotes: bool,
}

impl<'a> CommandLineIter<'a> {
    pub fn new(unicode: &'a UNICODE_STRING) -> Self {
        let data = if unicode.Buffer.is_null() || unicode.Length == 0 {
            &[]
        } else {
            unsafe {
                let len = (unicode.Length / 2) as usize;
                core::slice::from_raw_parts(unicode.Buffer, len)
            }
        };
        Self { data, pos: 0, in_quotes: false }
    }
}

impl<'a> Iterator for CommandLineIter<'a> {
    type Item = CommandLineArg<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.data.len() && is_whitespace(self.data[self.pos]) && !self.in_quotes {
            self.pos += 1;
        }

        if self.pos >= self.data.len() {
            return None;
        }

        let start = self.pos;
        let mut escaped = false;

        while self.pos < self.data.len() {
            let ch = self.data[self.pos];

            if escaped {
                escaped = false;
                self.pos += 1;
                continue;
            }

            match ch {
                0x22 => {
                    if self.in_quotes && self.pos + 1 < self.data.len() && self.data[self.pos + 1] == 0x22 {
                        self.pos += 2;
                    } else {
                        self.in_quotes = !self.in_quotes;
                        self.pos += 1;
                        if !self.in_quotes {
                            if self.pos < self.data.len() && is_whitespace(self.data[self.pos]) {
                                break;
                            }
                        }
                    }
                }
                0x5C => {
                    escaped = true;
                    self.pos += 1;
                }
                _ if is_whitespace(ch) && !self.in_quotes => {
                    break;
                }
                _ => {
                    self.pos += 1;
                }
            }
        }

        let end = self.pos;

        while self.pos < self.data.len() && is_whitespace(self.data[self.pos]) && !self.in_quotes {
            self.pos += 1;
        }

        if start == end {
            return None;
        }

        Some(CommandLineArg(&self.data[start..end]))
    }
}

fn is_whitespace(ch: u16) -> bool {
    ch == 0x20 || ch == 0x09 || ch == 0x0A || ch == 0x0D
}