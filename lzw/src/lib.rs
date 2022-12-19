mod trie;

use std::io;
use trie::{Trie, TrieWalker};

pub fn encode<R: io::Read, W: io::Write>(input: &mut R, output: &mut W) -> io::Result<()> {
    let mut dictionary = TrieWalker::new();
    let buffer = &mut [0; 1024];
    loop {
        let read = input.read(buffer)?;

        if read == 0 {
            output.write_all(&dictionary.get_last().to_be_bytes()[..])?;
            break;
        }

        for i in 0..read {
            if let Some(index) = dictionary.add_byte(buffer[i]) {
                log::info!("Writing {}", index);
                output.write_all(&index.to_be_bytes()[..])?;
            }
        }
    }

    Ok(())
}

pub fn decode<R: io::Read, W: io::Write>(input: &mut R, output: &mut W) -> io::Result<()> {
    let mut dictionary = Trie::new();
    let mut cur: Vec<u8> = Vec::new();
    let mut buf: [u8; std::mem::size_of::<usize>()] = [0; std::mem::size_of::<usize>()];

    loop {
        if let Err(error) = input.read_exact(&mut buf[..]) {
            match error.kind() {
                io::ErrorKind::UnexpectedEof => break,
                _ => return Err(error),
            }
        }

        let next = usize::from_be_bytes(buf);

        match dictionary.get_by_index(next) {
            Some(string) => {
                output.write_all(&string[..])?;

                cur.push(string[0]);
                dictionary.add(&cur[..]);
                cur = string;
            }
            None => {
                cur.push(cur[0]);
                dictionary.add(&cur[..]);

                output.write_all(&cur[..])?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoded_equal_to_data() {
        let data: &mut &[u8] = &mut &b"Hello, world"[..];
        let mut encoder_output = Vec::new();
        assert!(encode(data, &mut encoder_output).is_ok());
        let mut decoder_output = Vec::new();
        assert!(decode(&mut &encoder_output[..], &mut decoder_output).is_ok());
        assert_eq!(b"Hello, world", &decoder_output[..]);
    }
}
