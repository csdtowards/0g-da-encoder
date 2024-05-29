use super::error::VerifierError;
use crate::{
    amt::slice::EncodedSliceAMT, constants::G1Curve,
    merkle::slice::EncodedSliceMerkle, utils::scalar_to_h256, ZgSignerParams,
};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

#[derive(Debug, CanonicalSerialize, CanonicalDeserialize, PartialEq)]
pub struct EncodedSlice {
    pub index: usize,
    amt: EncodedSliceAMT,
    merkle: EncodedSliceMerkle,
}

impl EncodedSlice {
    pub(super) fn new(
        index: usize, amt: EncodedSliceAMT, merkle: EncodedSliceMerkle,
    ) -> Self {
        Self { index, amt, merkle }
    }

    pub fn verify(
        &self, encoder_amt: &ZgSignerParams,
        authoritative_commitment: &G1Curve, authoritative_root: &[u8; 32],
    ) -> Result<(), VerifierError> {
        // consistency between amt and merkle
        // index consistency
        if self.index != self.amt.index() {
            return Err(VerifierError::UnmatchedAMTIndex {
                row_index: self.index,
                amt_index: self.amt.index(),
            });
        }
        if self.index != self.merkle.index() {
            return Err(VerifierError::UnmatchedMerkleIndex {
                row_index: self.index,
                merkle_index: self.merkle.index(),
            });
        }
        // derive row_merkle from row_amt
        let row_amt = self.amt.row();
        let row_merkle: Vec<_> =
            row_amt.iter().map(|x| scalar_to_h256(*x)).collect();
        // verify amt, merkle
        self.amt.verify(encoder_amt, authoritative_commitment)?;
        self.merkle.verify(authoritative_root, row_merkle)?;
        Ok(())
    }
}
