/// Represents an ID3v2 synchsafe integer.
/// A synchsafe integer's bytes have 0 for their most significant bit to prevent
/// MP3 players mistaking tag data for an MP3 sync word (FFF).
/// See [Wikipedia](https://en.wikipedia.org/wiki/MP3#Design) for more detail.
#[derive(Clone, Debug)]
pub struct SynchSafeInteger {
    /// The bytes of the integer.
    pub bytes: Vec<u8>,
}

impl SynchSafeInteger {
    /// Returns a new SynchSafeInteger for the given bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The bytes of the integer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::synch_safe_integer::*;
    /// let bytes = [b'\x00', b'\x00', b'\x21', b'\x79'];
    ///
    /// let integer = SynchSafeInteger::new(&bytes);
    ///
    /// assert_eq!(u32::from(integer), 4345);
    /// ```
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
        }
    }

    pub fn from_5byte(value: u32) -> Self {
        let mask = 0b0111_1111u32;
        let bytes: Vec<u8> = (0..5)
            .map(|index| ((value >> (7 * index)) & mask) as u8)
            .rev()
            .collect();
        Self::new(&bytes)
    }
}

impl From<SynchSafeInteger> for u32 {
    fn from(item: SynchSafeInteger) -> Self {
        let mut result: u32 = 0;
        for (index, &byte) in item.bytes.iter().rev().enumerate() {
            let byte = u32::from(byte);
            result |= byte << (7 * index);
        }
        result
    }
}

impl From<u32> for SynchSafeInteger {
    /// # Examples
    ///
    /// ```
    /// # use sisko_lib::synch_safe_integer::*;
    /// let bytes = [b'\x00', b'\x00', b'\x21', b'\x79'];
    ///
    /// let integer = SynchSafeInteger::from(4345u32);
    ///
    /// assert_eq!(integer.bytes[0], b'\x00');
    /// assert_eq!(integer.bytes[1], b'\x00');
    /// assert_eq!(integer.bytes[2], b'\x21');
    /// assert_eq!(integer.bytes[3], b'\x79');
    /// ```
    fn from(value: u32) -> Self {
        let mask = 0b0111_1111u32;
        let bytes: Vec<u8> = (0..4)
            .map(|index| ((value >> (7 * index)) & mask) as u8)
            .rev()
            .collect();
        Self::new(&bytes)
    }
}
