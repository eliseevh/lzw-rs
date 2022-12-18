use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};

pub fn encode<R: Read, W: Write>(input: &mut R, output: &mut W) -> std::io::Result<()> {
    let mut dictionary = Trie::new();
    let mut cur: Vec<u8> = Vec::new();
    let mut last_index: usize = 0;
    let buffer = &mut [0; 1024];
    loop {
        let read = input.read(buffer)?;

        if read == 0 {
            log::info!("Writing {}", last_index);
            output.write_all(&last_index.to_be_bytes()[..])?;
            break;
        }

        for i in 0..read {
            cur.push(buffer[i]);
            if !dictionary.contains(&cur[..]) {
                log::info!("Writing {}", last_index);
                output.write_all(&last_index.to_be_bytes()[..])?;

                dictionary.add(&cur[..]);

                cur.clear();
                cur.push(buffer[i]);
            }
            last_index = dictionary.get(&cur[..]).unwrap();
        }
    }
    log::info!("Dictionary in the end is {:?}", dictionary);
    Ok(())
}

pub fn decode<R: Read, W: Write>(input: &mut R, output: &mut W) -> std::io::Result<()> {
    let mut dictionary = Trie::new();
    let mut cur: Vec<u8> = Vec::new();
    let mut buf: [u8; std::mem::size_of::<usize>()] = [0; std::mem::size_of::<usize>()];

    loop {
        if let Err(error) = input.read_exact(&mut buf[..]) {
            match error.kind() {
                ErrorKind::UnexpectedEof => break,
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

#[derive(Clone, Eq, PartialEq, Debug)]
struct Trie {
    tree: Vec<(usize, u8, HashMap<u8, usize>)>,
}

impl Trie {
    fn new() -> Self {
        let mut tree = Vec::with_capacity(257);
        tree.push((0, 0, HashMap::with_capacity(256)));

        for byte in 0..=255 {
            tree.push((0, byte, HashMap::new()));
            tree[0].2.insert(byte, byte as usize + 1);
        }

        Self { tree }
    }

    fn get_by_index(&self, index: usize) -> Option<Vec<u8>> {
        let mut result = Vec::new();
        let mut cur = index;
        while cur != 0 {
            let got = self.tree.get(cur)?;
            result.push(got.1);
            cur = got.0;
        }
        result.reverse();
        Some(result)
    }

    fn add(&mut self, bytes: &[u8]) {
        let mut cur = 0;
        for byte in bytes {
            if !self.tree[cur].2.contains_key(byte) {
                let len = self.tree.len();
                self.tree[cur].2.insert(*byte, len);
                self.tree.push((cur, *byte, HashMap::new()));
            }

            cur = self.tree[cur].2[byte];
        }
    }

    fn get(&self, bytes: &[u8]) -> Option<usize> {
        let mut cur = 0;
        for byte in bytes {
            cur = *self.tree[cur].2.get(byte)?;
        }
        Some(cur)
    }

    fn contains(&self, bytes: &[u8]) -> bool {
        self.get(bytes).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod trie {
        use super::*;

        #[test]
        fn trie_contains_one_char_strings() {
            let trie = Trie::new();
            for byte in 0..=255 {
                let slice = &[byte][..];
                assert!(trie.contains(slice));
            }
        }

        #[test]
        fn trie_contains_strings() {
            let mut trie = Trie::new();
            trie.add(b"Hello");
            trie.add(b"World");
            assert!(trie.contains(b"Hello"));
            assert!(!trie.contains(b"HelloWorld"));
        }

        #[test]
        fn trie_contains_prefix() {
            let mut trie = Trie::new();
            trie.add(b"Hello, world");
            assert!(trie.contains(b"Hello"));
            assert!(!trie.contains(b"world"));
        }

        #[test]
        fn trie_case_sensitive() {
            let mut trie = Trie::new();
            trie.add(b"Hello");
            assert!(!trie.contains(b"hello"))
        }
    }

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
