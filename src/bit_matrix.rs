use crate::{display::CELL_COUNT0, error::Error::BitsToIndexesNotEnoughSpace};
use core::{array, ops::BitOrAssign};

use heapless::{LinearMap, Vec};

use crate::{error::Error, leds::Leds};

#[derive(defmt::Format, Debug)]
pub struct BitMatrix([u8; CELL_COUNT0]);

impl BitMatrix {
    pub fn new(bits: [u8; CELL_COUNT0]) -> Self {
        Self(bits)
    }
    pub fn from_bits(bits: u8) -> Self {
        Self([bits; CELL_COUNT0])
    }

    pub fn iter(&self) -> impl Iterator<Item = &u8> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<u8> {
        self.0.iter_mut()
    }

    // If too long, turn on all decimal points
    pub fn from_str<S: AsRef<str>>(str: S) -> Self {
        let str = str.as_ref();

        let mut bit_matrix = BitMatrix::default();
        for (bits, c) in bit_matrix.iter_mut().zip(str.chars()) {
            *bits = Leds::ASCII_TABLE[c as usize];
        }

        if str.len() > CELL_COUNT0 {
            bit_matrix |= Leds::DECIMAL;
        }

        bit_matrix
    }

    pub fn from_chars(chars: &[char; CELL_COUNT0]) -> Self {
        let bytes = chars.map(|c| Leds::ASCII_TABLE[c as usize]);
        Self::new(bytes)
    }

    pub fn from_number(mut number: u16, padding: u8) -> Self {
        let mut bit_matrix = BitMatrix::from_bits(padding);

        for bits in bit_matrix.iter_mut().rev() {
            *bits = Leds::DIGITS[(number % 10) as usize]; // Get the last digit
            number /= 10; // Remove the last digit
            if number == 0 {
                break;
            }
        }
        // If the original number was out of range, turn on all decimal points
        if number > 0 {
            bit_matrix |= Leds::DECIMAL;
        }

        bit_matrix
    }

    pub fn bits_to_indexes(&self) -> Result<BitsToIndexes, Error> {
        let mut acc: BitsToIndexes = LinearMap::new();
        for (index, &bits) in self.iter().enumerate().filter(|(_, &bits)| bits != 0) {
            if let Some(vec) = acc.get_mut(&bits) {
                vec.push(index).map_err(|_| BitsToIndexesNotEnoughSpace)?;
            } else {
                let vec = Vec::from_slice(&[index]).map_err(|()| BitsToIndexesNotEnoughSpace)?;
                acc.insert(bits, vec)
                    .map_err(|_| BitsToIndexesNotEnoughSpace)?;
            }
        }
        Ok(acc)
    }
}

impl Default for BitMatrix {
    fn default() -> Self {
        Self([0; CELL_COUNT0])
    }
}

// Implement `|=` for `BitMatrix`
impl BitOrAssign<u8> for BitMatrix {
    fn bitor_assign(&mut self, rhs: u8) {
        self.0.iter_mut().for_each(|bits| *bits |= rhs);
    }
}

impl IntoIterator for BitMatrix {
    type Item = u8;
    type IntoIter = array::IntoIter<u8, CELL_COUNT0>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a BitMatrix {
    type Item = &'a u8;
    type IntoIter = core::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut BitMatrix {
    type Item = &'a mut u8;
    type IntoIter = core::slice::IterMut<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

// implement index for BitMatrix and &BitMatrix
impl core::ops::Index<usize> for BitMatrix {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

// index that you can assign to
impl core::ops::IndexMut<usize> for BitMatrix {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

// cmk move
type BitsToIndexes = LinearMap<u8, Vec<usize, CELL_COUNT0>, CELL_COUNT0>;
