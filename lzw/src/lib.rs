use std::collections::HashMap;
use std::io::{Read, Write};

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

#[derive(Clone, Eq, PartialEq, Debug)]
struct Trie {
    tree: Vec<HashMap<u8, usize>>,
}

impl Trie {
    fn new() -> Self {
        let mut tree = Vec::with_capacity(257);
        tree.push(HashMap::with_capacity(256));

        for byte in 0..=255 {
            tree.push(HashMap::new());
            tree[0].insert(byte, byte as usize + 1);
        }

        Self { tree }
    }

    fn add(&mut self, bytes: &[u8]) {
        let mut cur = 0;
        for byte in bytes {
            if !self.tree[cur].contains_key(byte) {
                let len = self.tree.len();
                self.tree[cur].insert(*byte, len);
                self.tree.push(HashMap::new());
            }

            cur = self.tree[cur][byte];
        }
    }

    fn get(&self, bytes: &[u8]) -> Option<usize> {
        let mut cur = 0;
        for byte in bytes {
            cur = *self.tree[cur].get(byte)?;
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
}
