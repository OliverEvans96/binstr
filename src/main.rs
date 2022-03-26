use std::{
    io::{Cursor, Read, Write},
    str::Utf8Error,
};

use anyhow::bail;
use clap::Parser;
use derive_more::Deref;

const BYTE_SIZE: usize = 8;

#[derive(Debug, Parser)]
struct Opts {
    /// Decode from digits to string
    #[clap(short = 'd')]
    decode: bool,

    /// No trailing newline in output
    #[clap(short = 'n')]
    no_trailing_newline: bool,

    /// Don't strip trailing newline from input
    #[clap(long = "no-strip")]
    no_strip: bool,
}

/// Read everything up front & trim newline from end
struct TrimmedOneTimeReader(Cursor<Vec<u8>>);

/// Remove trailing \n and \r from byte slice
fn strip_trailing_whitespace(v: &mut Vec<u8>) {
    let n = '\n' as u8;
    let r = '\n' as u8;
    while v.ends_with(&[n]) || v.ends_with(&[r]) {
        v.pop();
    }
}

impl TrimmedOneTimeReader {
    pub fn try_new<R: Read>(mut reader: R) -> std::io::Result<Self> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        strip_trailing_whitespace(&mut buf);
        Ok(Self(Cursor::new(buf)))
    }
}

impl Read for TrimmedOneTimeReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.0.read(buf)?;
        Ok(n)
    }
}

#[derive(Clone, Debug, Deref, Eq, PartialEq)]
struct BinMsg(Vec<u8>);

#[derive(Clone, Debug, Deref, Eq, PartialEq)]
struct StrMsg(String);

impl BinMsg {
    pub fn read(mut reader: impl Read) -> anyhow::Result<BinMsg> {
        let mut strbuf = String::new();
        reader.read_to_string(&mut strbuf)?;

        let slen = strbuf.len();
        if slen % BYTE_SIZE != 0 {
            bail!(
                "Number of binary digits must be a multiple of {}, got {}.",
                BYTE_SIZE,
                slen
            );
        }

        let all_bits = str_to_bits(&strbuf)?;
        let bit_arrs = bit_slice_to_arrays(&all_bits);
        let bytes: Vec<u8> = bit_arrs
            .into_iter()
            .map(|bits| bits_to_byte(bits))
            .collect();

        Ok(BinMsg(bytes))
    }

    pub fn write(&self, mut writer: impl Write) -> anyhow::Result<()> {
        // let s = self.iter().map(|x|);
        // writer.write_all(self.as_bytes())?;
        let bits: Vec<bool> = self.iter().flat_map(|&b| byte_to_bits(b)).collect();
        let s = bits_to_str(&bits);
        writer.write_all(s.as_bytes())?;

        Ok(())
    }
}

fn str_to_bits(s: &str) -> anyhow::Result<Vec<bool>> {
    let mut binvec = Vec::new();
    for c in s.chars() {
        match c {
            '1' => binvec.push(true),
            '0' => binvec.push(false),
            other => bail!("Encountered non-binary digit {:?}", other),
        }
    }
    Ok(binvec)
}

fn bits_to_str(bits: &[bool]) -> String {
    bits.iter()
        .map(|&bit| if bit { '1' } else { '0' })
        .collect()
}

fn bit_slice_to_arrays(slice: &[bool]) -> Vec<[bool; BYTE_SIZE]> {
    slice
        .chunks_exact(BYTE_SIZE)
        .map(|chunk| {
            let mut arr = [false; BYTE_SIZE];
            arr.copy_from_slice(chunk);
            arr
        })
        .collect()
}

fn bits_to_byte(bits: [bool; 8]) -> u8 {
    bits.into_iter()
        .rev()
        .enumerate()
        .fold(0u8, |acc, (i, bit)| acc | ((bit as u8) << i))
}

fn byte_to_bits(byte: u8) -> [bool; BYTE_SIZE] {
    let bits_iter = (0..BYTE_SIZE).rev().map(|i| byte & (1 << i) != 0);

    let mut bits_arr = [false; BYTE_SIZE];
    for (i, bit) in bits_iter.enumerate() {
        bits_arr[i] = bit;
    }

    bits_arr
}

impl StrMsg {
    pub fn read(mut reader: impl Read) -> anyhow::Result<StrMsg> {
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        Ok(Self(buf))
    }

    pub fn write(&self, mut writer: impl Write) -> anyhow::Result<()> {
        writer.write_all(self.as_bytes())?;
        Ok(())
    }
}

impl TryFrom<BinMsg> for StrMsg {
    type Error = Utf8Error;

    fn try_from(b: BinMsg) -> Result<Self, Self::Error> {
        let s = std::str::from_utf8(&*b)?;
        Ok(Self(s.to_owned()))
    }
}

impl From<StrMsg> for BinMsg {
    fn from(s: StrMsg) -> Self {
        let b = s.as_bytes();
        Self(b.to_owned())
    }
}

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    let input: Box<dyn Read> = if opts.no_strip {
        Box::new(stdin)
    } else {
        Box::new(TrimmedOneTimeReader::try_new(stdin)?)
    };

    if opts.decode {
        decode(input, &mut stdout)?;
    } else {
        encode(input, &mut stdout)?;
    }

    if !opts.no_trailing_newline {
        writeln!(stdout, "")?;
    }

    Ok(())
}

fn encode(reader: impl Read, writer: impl Write) -> anyhow::Result<()> {
    let smsg = StrMsg::read(reader)?;
    let bmsg: BinMsg = smsg.into();
    bmsg.write(writer)?;

    Ok(())
}

fn decode(reader: impl Read, writer: impl Write) -> anyhow::Result<()> {
    let bmsg = BinMsg::read(reader)?;
    let smsg: StrMsg = bmsg.try_into()?;
    smsg.write(writer)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn test_str_to_bits() {
        let s = "10010101";
        let bits = str_to_bits(s).unwrap();
        let expected = vec![true, false, false, true, false, true, false, true];
        assert_eq!(bits, expected);
    }

    #[test_case(vec![true, true, false], "110")]
    #[test_case(vec![true, false, false, true], "1001")]
    fn test_bits_to_str(bits: Vec<bool>, s: &str) {
        assert_eq!(bits_to_str(&bits), s);
    }

    #[test]
    fn test_bit_slice_to_arrays() {
        let slice = vec![
            true, false, true, true, true, false, true, true, false, true, false, true, false,
            true, true, true,
        ];
        let arrays = vec![
            [true, false, true, true, true, false, true, true],
            [false, true, false, true, false, true, true, true],
        ];
        assert_eq!(bit_slice_to_arrays(&slice), arrays);
    }

    #[test_case("00000000", 0b0)]
    #[test_case("00000001", 0b1)]
    #[test_case("00000010", 0b10)]
    #[test_case("00000011", 0b11)]
    fn test_bits_to_byte(s: &str, b: u8) {
        let bits = str_to_bits(s).unwrap();
        let mut bits_arr: [bool; BYTE_SIZE] = [false; BYTE_SIZE];
        bits_arr.copy_from_slice(&bits);
        let byte = bits_to_byte(bits_arr);
        assert_eq!(byte, b);
    }

    #[test_case("00000000", 0b0)]
    #[test_case("00000001", 0b1)]
    #[test_case("00000010", 0b10)]
    #[test_case("00000011", 0b11)]
    fn test_byte_to_bits(s: &str, b: u8) {
        let sbits = str_to_bits(s).unwrap();
        let bbits = byte_to_bits(b);

        assert_eq!(&sbits, &bbits);
    }

    #[test_case("0000000000000000", vec![0b0, 0b0])]
    #[test_case("0000000100000010", vec![0b1, 0b10])]
    fn test_read_write_binmsg(s: &str, b: Vec<u8>) {
        let msg = BinMsg::read(s.as_bytes()).unwrap();
        assert_eq!(msg.0, b);

        let mut buf = Vec::new();
        msg.write(&mut buf).unwrap();
        let sfinal = std::str::from_utf8(&buf).unwrap();
        assert_eq!(s, sfinal);
    }

    #[test_case("hello")]
    #[test_case("1 n2 m432,m654"; "arbitrary string")]
    #[test_case("😂 I'm d"; "with emojis")]
    #[test_case("終於有了信號 我沒"; "chinese")]
    fn test_read_write_strmsg(s: &str) {
        let msg = StrMsg::read(s.as_bytes()).unwrap();
        assert_eq!(&*msg, s);

        let mut buf = Vec::new();
        msg.write(&mut buf).unwrap();
        let sfinal = std::str::from_utf8(&buf).unwrap();
        assert_eq!(s, sfinal);
    }

    #[test_case("01100001", "a")]
    #[test_case("01100010", "b")]
    #[test_case("0110000101100010", "ab")]
    fn test_smsg_bmsg_conversion(b: &str, s: &str) {
        let bmsg = BinMsg::read(b.as_bytes()).unwrap();
        let smsg = StrMsg::read(s.as_bytes()).unwrap();
        assert_eq!(bmsg, smsg.clone().into());
        assert_eq!(bmsg.try_into(), Ok(smsg));
    }
}
