use crate::{
    cfg_chunks_exact,
    constants::{Scalar, MAX_BLOB_SIZE, RAW_BLOB_SIZE, RAW_UNIT},
    raw_data::RawData,
    scalar_to_h256,
    utils::raw_unit_to_scalar,
};
use std::ops::{Deref, DerefMut};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct RawBlob(Vec<Scalar>); // BLOB_ROW_N * BLOB_COL_N

impl RawBlob {
    pub fn new(input: Vec<Scalar>) -> Self {
        assert_eq!(input.len(), RAW_BLOB_SIZE);
        Self(input)
    }
}

impl Deref for RawBlob {
    type Target = [Scalar];

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for RawBlob {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl From<RawData> for RawBlob {
    fn from(data: RawData) -> Self {
        let raw_blob_1d: Vec<_> = cfg_chunks_exact!(data, RAW_UNIT)
            .map(raw_unit_to_scalar)
            .collect();
        RawBlob(raw_blob_1d)
    }
}

impl TryFrom<RawBlob> for RawData {
    type Error = String;

    fn try_from(blob: RawBlob) -> Result<RawData, String> {
        let bytes32_list: Vec<[u8; 32]> =
            cfg_iter!(blob.0).cloned().map(scalar_to_h256).collect();
        let mut raw_data = Vec::with_capacity(MAX_BLOB_SIZE);
        for bytes32 in bytes32_list.into_iter() {
            if bytes32[31] != 0 {
                return Err("Incorrect scalar".to_string());
            }
            raw_data.extend_from_slice(&bytes32[..31]);
        }

        RawData::try_from_padded(raw_data)
    }
}

#[test]
fn test_raw_blob_recover() {
    use crate::constants::MAX_RAW_DATA_SIZE;
    use rand::{thread_rng, RngCore};

    let test_case = |input: Vec<u8>| {
        let raw_data: RawData = input.as_slice().try_into().unwrap();
        let raw_blob: RawBlob = raw_data.clone().into();

        let raw_data_recover: RawData = raw_blob.try_into().unwrap();
        let input_recover = raw_data_recover.as_bytes().to_vec();

        assert_eq!(raw_data, raw_data_recover);
        assert_eq!(input_recover, input);
    };

    let mut rng = thread_rng();

    for i in (0usize..=MAX_RAW_DATA_SIZE).step_by(4) {
        let mut input = vec![0; i];
        rng.fill_bytes(&mut input[..]);

        test_case(input);
    }
}
