use std::collections::HashMap;

const MAX_SIZE: usize = 64 * 1024 * 1024;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Trie {
    tree: Vec<(usize, Vec<u8>, HashMap<u8, usize>)>,
}

impl Trie {
    pub fn new() -> Self {
        let mut tree = Vec::with_capacity(257);
        tree.push((0, vec![], HashMap::with_capacity(256)));

        for byte in 0..=255 {
            tree.push((0, vec![byte], HashMap::new()));
            tree[0].2.insert(byte, byte as usize + 1);
        }

        Self { tree }
    }

    pub fn get_by_index(&self, index: usize) -> Option<Vec<u8>> {
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

    pub fn len(&self) -> usize {
        self.tree.len()
    }

    fn add_byte(&mut self, index: usize, byte: u8) -> bool {
        if !self.tree[index].2.contains_key(&byte) {
            let len = self.tree.len();
            if len < MAX_SIZE {
                self.tree[index].2.insert(byte, len as usize);
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

    #[cfg(test)]
    fn get(&self, bytes: &[u8]) -> Option<usize> {
        let mut cur = 0;
        for byte in bytes {
            cur = *self.tree[cur as usize].2.get(byte)?;
        }
        Some(cur)
    }

    #[cfg(test)]
    fn contains(&self, bytes: &[u8]) -> bool {
        self.get(bytes).is_some()
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TrieWalker {
    trie: Trie,
    index: usize,
}

impl TrieWalker {
    pub fn new() -> Self {
        Self {
            trie: Trie::new(),
            index: 0,
        }
    }

    pub fn add_byte(&mut self, byte: u8) -> Option<usize> {
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

    pub fn get_last(self) -> usize {
        self.index
    }

    pub fn len(&self) -> usize {
        self.trie.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_one_char_strings() {
        let trie = Trie::new();
        for byte in 0..=255 {
            let slice = &[byte][..];
            assert!(trie.contains(slice));
        }
    }

    #[test]
    fn contains_strings() {
        let mut trie = Trie::new();
        trie.add(b"Hello");
        trie.add(b"World");
        assert!(trie.contains(b"Hello"));
        assert!(!trie.contains(b"HelloWorld"));
    }

    #[test]
    fn contains_prefix() {
        let mut trie = Trie::new();
        trie.add(b"Hello, world");
        assert!(trie.contains(b"Hello"));
        assert!(!trie.contains(b"world"));
    }

    #[test]
    fn case_sensitive() {
        let mut trie = Trie::new();
        trie.add(b"Hello");
        assert!(!trie.contains(b"hello"))
    }
}
