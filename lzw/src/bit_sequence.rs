use std::io;

const fn log_2(a: usize) -> usize {
    let mut result = 0;
    let mut power = 1;
    while power < a {
        power *= 2;
        result += 1;
    }
    result
}

const BYTE_SIZE: usize = 8;

type Element = u64;
const ELEMENT_SIZE: usize = std::mem::size_of::<Element>() * BYTE_SIZE;
const LOG_ELEMENT_SIZE: usize = log_2(ELEMENT_SIZE);
const ONES: Element = Element::MAX;

pub struct BitSequence {
    data: Vec<u64>,
    size: usize,
}

impl BitSequence {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            size: 0,
        }
    }

    pub fn add_number(&mut self, number: u64, number_size: usize) {
        if number_size == 0 {
            return;
        }
        let free_size = (ELEMENT_SIZE - (self.size % ELEMENT_SIZE)) % ELEMENT_SIZE;
        if free_size >= number_size {
            let last_idx = self.data.len() - 1;

            let mask = ONES << (ELEMENT_SIZE - free_size);
            let needed_part = mask & (self.data[last_idx]);
            let xor_mask = (number << (ELEMENT_SIZE - free_size)) ^ needed_part;

            self.data[last_idx] ^= xor_mask;
            self.size += number_size;
        } else {
            self.add_number(number, free_size);
            self.data.push(number >> free_size);
            self.size += number_size - free_size;
        }
    }

    pub fn get_number(&self, number_size: usize, start_position: usize) -> Element {
        if number_size == 0 {
            return 0;
        }

        let taken_size = ELEMENT_SIZE - start_position % ELEMENT_SIZE;

        if taken_size >= number_size {
            (self.data[start_position >> LOG_ELEMENT_SIZE] >> (start_position % ELEMENT_SIZE))
                & (ONES >> ELEMENT_SIZE - number_size)
        } else {
            let lower = self.get_number(taken_size, start_position);
            let higher = self.get_number(number_size - taken_size, start_position + taken_size);

            lower | (higher << taken_size)
        }
    }

    pub fn dump_current<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        for i in 0..self.size / BYTE_SIZE {
            writer.write_all(&[self.get_number(BYTE_SIZE, i * BYTE_SIZE) as u8])?;
        }
        let mut new = BitSequence::new();
        let left_size = self.size % BYTE_SIZE;
        new.add_number(self.get_number(left_size, self.size - left_size), left_size);
        *self = new;
        Ok(())
    }

    pub fn dump_end<W: io::Write>(mut self, writer: &mut W) -> io::Result<()> {
        let empty_size = (BYTE_SIZE - self.size % BYTE_SIZE) % BYTE_SIZE;
        self.add_number(0, empty_size);
        self.dump_current(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_default() {
        let empty = BitSequence::new();
        assert_eq!(empty.size, 0);
    }

    #[test]
    fn store_values() {
        let mut bit_sequence = BitSequence::new();
        let mut values = Vec::new();

        for i in 0..1000 {
            values.push(i);
        }

        for value in &values {
            bit_sequence.add_number(*value, 16);
        }

        assert_eq!(bit_sequence.size, 16 * 1000);
        for i in 0..1000 {
            assert_eq!(bit_sequence.get_number(16, 16 * i), values[i]);
        }
    }

    #[test]
    fn store_different_size_values() {
        let mut bit_sequence = BitSequence::new();
        let mut values = Vec::new();

        for i in 0..1000 {
            values.push(i);
        }

        for i in 0..100 {
            bit_sequence.add_number(values[i], 7);
        }
        for i in 100..500 {
            bit_sequence.add_number(values[i], 9);
        }
        for i in 500..1000 {
            bit_sequence.add_number(values[i], 30);
        }

        assert_eq!(
            bit_sequence.size,
            100 * 7 + (500 - 100) * 9 + (1000 - 500) * 30,
        );
        for i in 0..100 {
            assert_eq!(bit_sequence.get_number(7, 7 * i), values[i]);
        }
        for i in 100..500 {
            assert_eq!(
                bit_sequence.get_number(9, 7 * 100 + (i - 100) * 9),
                values[i]
            );
        }
        for i in 500..1000 {
            assert_eq!(
                bit_sequence.get_number(30, 7 * 100 + (500 - 100) * 9 + (i - 500) * 30),
                values[i]
            );
        }
    }

    #[test]
    fn dump_byte_divisible() {
        let mut bit_sequence = BitSequence::new();
        let mut values: Vec<u8> = Vec::new();

        for i in 0..1000 {
            values.push((i % 256) as u8);
        }

        for value in &values {
            bit_sequence.add_number(*value as u64, 8);
        }

        let mut dump: Vec<u8> = Vec::new();
        assert!(bit_sequence.dump_end(&mut dump).is_ok());

        assert_eq!(dump, values);
    }

    #[test]
    fn dump_non_byte_divisible() {
        let mut bit_sequence = BitSequence::new();
        let mut values: Vec<u8> = Vec::new();

        for _ in 0..500 {
            values.push(0b11111); // 0b11111, so bit sequence will be filled with ones
        }

        for value in &values {
            bit_sequence.add_number(*value as u64, 5);
        }

        let mut dump: Vec<u8> = Vec::new();
        assert!(bit_sequence.dump_current(&mut dump).is_ok());

        assert_eq!(dump.len(), (500 * 5) / 8);
        assert_eq!(bit_sequence.size, (500 * 5) % 8);
        for value in dump {
            assert_eq!(value, 0b11111111);
        }

        // there are 4 ones in bit sequence, and we adding 9 more, so there are one byte + 5 ones
        bit_sequence.add_number(0b1111_11111, 9);

        let mut dump: Vec<u8> = Vec::new();
        assert!(bit_sequence.dump_end(&mut dump).is_ok());

        assert_eq!(dump.len(), 2);
        assert_eq!(dump[0], 0b11111111);
        assert_eq!(dump[1], 0b00011111);
    }
}
