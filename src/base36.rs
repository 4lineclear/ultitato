use std::fmt::Display;

// TODO testing
use rand::{distributions::Uniform, prelude::Distribution, thread_rng};

pub const NUMERALS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
pub const MIN: u32 = 36u32.pow(5);
pub const MAX: u32 = 36u32.pow(6);

#[derive(Debug)]
pub struct UniformID(Uniform<u32>);

impl Default for UniformID {
    fn default() -> Self {
        Self(Uniform::from(MIN..MAX))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GameID(pub String);

impl GameID {
    pub fn new_rand(gen: &UniformID) -> Self {
        Self(encode(gen.0.sample(&mut thread_rng())))
    }
}

// TODO do actual parsing
impl From<String> for GameID {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Display for GameID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub fn encode(mut n: u32) -> String {
    if n < 10 {
        n.to_string()
    } else {
        let mut s = Vec::new();
        while 0 < n {
            s.push(NUMERALS[(n % 36) as usize]);
            n /= 36;
        }
        // SAFETY: NUMERALS is valid utf8
        unsafe { String::from_utf8_unchecked(s.into_iter().rev().collect()) }
    }
}

pub fn decode(s: &str) -> Option<u32> {
    s.as_bytes()
        .iter()
        .rev()
        .enumerate()
        .try_fold(0u32, |acc, (i, b)| match *b {
            b'0'..=b'9' => Some({
                let n = (b - b'0') as u32;
                let e = 36u32.pow(i as u32);
                acc + n * e
            }),
            b'a'..=b'z' => Some({
                let n = (b - b'a' + 10) as u32;
                let e = 36u32.pow(i as u32);
                acc + n * e
            }),
            _ => None,
        })
}
