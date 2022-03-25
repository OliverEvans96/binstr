use std::io::{BufReader, Read, Stdin};

use anyhow::bail;
use clap::Parser;

const BYTE_SIZE: usize = 8;

#[derive(Debug, Parser)]
struct Opts {
    /// Decode from digits to string
    #[clap(short = 'd')]
    decode: bool,
}

#[derive(Debug)]
struct BinMsg(Vec<u8>);

#[derive(Debug)]
struct StrMsg(String);

impl BinMsg {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<BinMsg> {
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

        let all_bits = chars_to_bits(strbuf.chars())?;
        let bit_arrs = bit_slice_to_arrays(&all_bits);
        let bytes: Vec<u8> = bit_arrs.into_iter().map(|bits|
            bits_to_byte(bits)
        ).collect();

        Ok(BinMsg(bytes))
    }
}

fn chars_to_bits<I>(chars: I) -> anyhow::Result<Vec<bool>>
where
    I: Iterator<Item = char>,
{
    let mut binvec = Vec::new();
    for c in chars {
        match c {
            '1' => binvec.push(true),
            '0' => binvec.push(false),
            other => bail!("Encountered non-binary digit {:?}", other),
        }
    }
    Ok(binvec)
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

impl StrMsg {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<StrMsg> {
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        Ok(Self(buf))
    }
}

impl From<BinMsg> for StrMsg {
    fn from(b: BinMsg) -> Self {
        let mut s = String::new();
        todo!()
    }
}

impl From<StrMsg> for BinMsg {
    fn from(s: StrMsg) -> Self {
        todo!()
    }
}

fn main() {
    let opts: Opts = Opts::parse();

    let mut stdin = std::io::stdin();

    // match opts.from {
    //     Format::Raw => todo!(),
    //     Format::Digits => todo!(),
    //     Format::String => todo!(),
    // }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use test_case::test_case;

    #[test]
    fn test_chars_to_bits() {
        let s = "10010101";
        let chars = s.chars();
        let bits = chars_to_bits(chars).unwrap();
        let expected = vec![true, false, false, true, false, true, false, true];
        assert_eq!(bits, expected);
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
        let bits = chars_to_bits(s.chars()).unwrap();
        let mut bits_arr: [bool; BYTE_SIZE] = [false; BYTE_SIZE];
        bits_arr.copy_from_slice(&bits);
        let byte = bits_to_byte(bits_arr);
        assert_eq!(byte, b);
    }

    #[test_case("0000000000000000", vec![0b0, 0b0])]
    #[test_case("0000000100000010", vec![0b1, 0b10])]
    fn test_read_binmsg(s: &str, b: Vec<u8>) {
        let mut sbytes = s.as_bytes();
        let msg = BinMsg::read(&mut sbytes).unwrap();
        assert_eq!(msg.0, b)
    }
}
