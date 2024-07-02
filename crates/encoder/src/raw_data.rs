use crate::{
    constants::{MAX_BLOB_SIZE, MAX_RAW_DATA_SIZE},
    encoder::error::EncoderError,
};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawData(Vec<u8>); // MAX_BLOB_SIZE

impl RawData {
    pub fn as_bytes(&self) -> &[u8] {
        let length = Self::get_actual_length(&self.0).unwrap();
        &self.0[..length]
    }

    pub(crate) fn try_from_padded(input: Vec<u8>) -> Result<Self, String> {
        if input.len() != MAX_BLOB_SIZE {
            return Err("Incorrect input length".to_string());
        }

        Self::get_actual_length(&input)?;
        Ok(Self(input))
    }

    fn get_actual_length(input: &[u8]) -> Result<usize, String> {
        let mut raw_length = [0u8; 4];
        raw_length.copy_from_slice(&input[MAX_RAW_DATA_SIZE..]);
        let length = u32::from_le_bytes(raw_length) as usize;
        if length > MAX_RAW_DATA_SIZE {
            return Err("Incorrect length field".to_string());
        }

        if input[length..MAX_RAW_DATA_SIZE].iter().any(|x| *x != 0) {
            return Err("Non zero in padding range".to_string());
        }

        Ok(length)
    }
}

impl Default for RawData {
    fn default() -> Self { RawData(vec![0u8; MAX_BLOB_SIZE]) }
}

impl Deref for RawData {
    type Target = [u8];

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for RawData {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl TryFrom<&[u8]> for RawData {
    type Error = EncoderError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let value_len = value.len();
        if value_len <= MAX_RAW_DATA_SIZE {
            let mut array = vec![0u8; MAX_BLOB_SIZE];
            array[..value_len].copy_from_slice(value);
            array[MAX_RAW_DATA_SIZE..]
                .copy_from_slice(&(value_len as u32).to_le_bytes());
            Ok(RawData(array))
        } else {
            Err(EncoderError::TooLargeBlob {
                actual: value.len(),
                expected_max: MAX_RAW_DATA_SIZE,
            })
        }
    }
}
