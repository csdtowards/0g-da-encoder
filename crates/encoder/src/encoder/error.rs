pub use crate::{amt::error::AmtError, merkle::error::MerkleError};

#[derive(Debug, PartialEq, Eq)]
pub enum EncoderError {
    TooLargeBlob { actual: usize, expected_max: usize },
}

#[derive(Debug, PartialEq, Eq)]
pub enum VerifierError {
    AMT(AmtError),
    Merkle(MerkleError),
    UnmatchedAMTIndex {
        row_index: usize,
        amt_index: usize,
    },
    UnmatchedMerkleIndex {
        row_index: usize,
        merkle_index: usize,
    },
}

impl Into<String> for EncoderError {
    fn into(self) -> String {
        format!("{:?}", self)
    }
}

impl From<AmtError> for VerifierError {
    fn from(error: AmtError) -> Self {
        VerifierError::AMT(error)
    }
}

impl From<MerkleError> for VerifierError {
    fn from(error: MerkleError) -> Self {
        VerifierError::Merkle(error)
    }
}
