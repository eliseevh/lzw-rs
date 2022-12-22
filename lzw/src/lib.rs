mod bit_sequence;
mod trie;

use bit_sequence::{log_2, BitSequence, Element};
use std::io;
use trie::{Trie, TrieWalker};

const BIT_SEQUENCE_BUFFER_SIZE: usize = 1024 * 1024 * 8;

pub fn encode<R: io::Read, W: io::Write>(input: &mut R, output: &mut W) -> io::Result<()> {
    let mut dictionary = TrieWalker::new();
    let mut bit_sequence = BitSequence::new();
    let buffer = &mut [0; BIT_SEQUENCE_BUFFER_SIZE / 8];
    loop {
        let read = input.read(buffer)?;

        if read == 0 {
            let len = log_2(dictionary.len());
            bit_sequence.add_number(dictionary.get_last() as Element - 1, len);
            bit_sequence.dump_end(output)?;
            break;
        }

        for i in 0..read {
            if let Some(index) = dictionary.add_byte(buffer[i]) {
                bit_sequence.add_number(index as Element - 1, log_2(dictionary.len() - 1));
                if bit_sequence.len() > BIT_SEQUENCE_BUFFER_SIZE {
                    bit_sequence.dump_current(output)?;
                }
            }
        }
    }

    Ok(())
}

pub fn decode<R: io::Read, W: io::Write>(input: &mut R, output: &mut W) -> io::Result<()> {
    let mut dictionary = Trie::new();
    let mut bit_sequence = BitSequence::new();
    let mut bit_sequence_position = 0;
    let mut cur: Vec<u8> = Vec::new();
    let mut buf: [u8; std::mem::size_of::<Element>()] = [0; std::mem::size_of::<Element>()];
    let mut can_read = true;

    while can_read {
        bit_sequence.cut(bit_sequence_position);
        bit_sequence_position = 0;
        while bit_sequence.len() < BIT_SEQUENCE_BUFFER_SIZE {
            let read = input.read(&mut buf)?;
            if read == 0 {
                can_read = false;
                break;
            }
            bit_sequence.add_number(Element::from_le_bytes(buf), read * 8);
        }
        while bit_sequence.len() - bit_sequence_position >= log_2(dictionary.len() + 1) {
            let cur_len = log_2(dictionary.len() + 1);
            let next = bit_sequence.get_number(cur_len, bit_sequence_position) as usize + 1;
            bit_sequence_position += cur_len;

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
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn decoded_equal_to_data() {
        let data: &mut &[u8] = &mut &b"Hello, world"[..];
        let mut encoder_output = Vec::new();
        assert!(encode(data, &mut encoder_output).is_ok());
        let mut decoder_output = Vec::new();
        assert!(decode(&mut &encoder_output[..], &mut decoder_output).is_ok());
        assert_eq!(b"Hello, world", &decoder_output[..]);
    }

    #[test]
    fn big_file() {
        let mut encoder_input =
            io::BufReader::new(std::fs::File::open("../test_data/war_and_peace.txt").unwrap());
        let mut encoder_output = io::BufWriter::new(
            std::fs::File::create("../test_data/war_and_peace.txt.compress").unwrap(),
        );

        assert!(encode(&mut encoder_input, &mut encoder_output).is_ok());

        drop(encoder_output);
        drop(encoder_input);

        let mut decoder_input = io::BufReader::new(
            std::fs::File::open("../test_data/war_and_peace.txt.compress").unwrap(),
        );
        let mut decoder_output = io::BufWriter::new(
            std::fs::File::create("../test_data/war_and_peace.txt.copy").unwrap(),
        );

        assert!(decode(&mut decoder_input, &mut decoder_output).is_ok());

        drop(decoder_input);
        drop(decoder_output);

        let mut original = std::fs::File::open("../test_data/war_and_peace.txt").unwrap();
        let mut copy = std::fs::File::open("../test_data/war_and_peace.txt.copy").unwrap();
        let mut original_read = Vec::new();
        let mut copy_read = Vec::new();

        assert!(original.read_to_end(&mut original_read).is_ok());
        assert!(copy.read_to_end(&mut copy_read).is_ok());

        assert_eq!(copy_read, original_read);
    }
}
