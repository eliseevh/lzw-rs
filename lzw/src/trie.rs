use std::collections::HashMap;

pub type Code = u16;
const MAX_SIZE: usize = (2 as usize).pow(8 * std::mem::size_of::<Code>() as u32);

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Trie {
    tree: Vec<(usize, Vec<u8>, HashMap<u8, Code>)>,
}

impl Trie {
    pub fn new() -> Self {
        let mut tree = Vec::with_capacity(257);
        tree.push((0, vec![], HashMap::with_capacity(256)));

        for byte in 0..=255 {
            tree.push((0, vec![byte], HashMap::new()));
            tree[0].2.insert(byte, byte as Code + 1);
        }

        Self { tree }
    }

    pub fn get_by_index(&self, index: Code) -> Option<Vec<u8>> {
        Some(self.tree.get(index as usize)?.1.clone())
    }

    pub fn add(&mut self, bytes: &[u8]) {
        let mut cur = 0;
        for byte in bytes {
            if self.add_byte(cur as usize, *byte) {
                cur = self.tree[cur as usize].2[byte];
            } else {
                break;
            }
        }
    }

    fn add_byte(&mut self, index: usize, byte: u8) -> bool {
        if !self.tree[index].2.contains_key(&byte) {
            let len = self.tree.len();
            if len < MAX_SIZE {
                self.tree[index].2.insert(byte, len as Code);
                let mut new = self.tree[index].1.clone();
                new.push(byte);
                self.tree.push((index, new, HashMap::new()));
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    fn get(&self, bytes: &[u8]) -> Option<Code> {
        let mut cur = 0;
        for byte in bytes {
            cur = *self.tree[cur as usize].2.get(byte)?;
        }
        Some(cur)
    }

    fn contains(&self, bytes: &[u8]) -> bool {
        self.get(bytes).is_some()
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TrieWalker {
    trie: Trie,
    index: Code,
}

impl TrieWalker {
    pub fn new() -> Self {
        Self {
            trie: Trie::new(),
            index: 0,
        }
    }

    pub fn add_byte(&mut self, byte: u8) -> Option<Code> {
        if !self.trie.tree[self.index as usize].2.contains_key(&byte) {
            self.trie.add_byte(self.index as usize, byte);
            let index = self.index;
            self.index = self.trie.tree[0].2[&byte];
            Some(index)
        } else {
            self.index = self.trie.tree[self.index as usize].2[&byte];
            None
        }
    }

    pub fn get_last(self) -> Code {
        self.index
    }
}

#[cfg(test)]
mod tests {
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
