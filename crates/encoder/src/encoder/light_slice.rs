use amt::{BlobRow, Proof};
use crate::{constants::{G1Curve, Scalar, BLOB_COL_LOG, BLOB_ROW_LOG, BLOB_ROW_N, COSET_N, PE}, merkle::Bytes32, EncodedSlice, EncodedSliceAMT, EncodedSliceMerkle};

pub struct LightEncodedSlice {
    index: usize,
    amt_commitment: G1Curve,
    amt_proof: Proof<PE>,
    amt_high_commitment: G1Curve,
    merkle_root: [Bytes32; COSET_N],
    merkle_proof: Vec<Bytes32>,
    merkle_leaf: Bytes32,
}

impl LightEncodedSlice {
    pub(crate) fn into_slice(&self, row: Vec<Scalar>) -> EncodedSlice {
        let amt_row = BlobRow::<PE, BLOB_COL_LOG, BLOB_ROW_LOG> {
            index: self.index % BLOB_ROW_N,
            row,
            proof: self.amt_proof.clone(),
            high_commitment: self.amt_high_commitment
        };
        let amt = EncodedSliceAMT::new(self.index, self.amt_commitment, amt_row);
        let merkle = EncodedSliceMerkle::new(self.merkle_root, self.merkle_proof.clone(), self.index, self.merkle_leaf);
        EncodedSlice::new(self.index, amt, merkle)
    }
}

impl LightEncodedSlice {
    fn new(
        index: usize,
        amt_commitment: G1Curve,
        amt_proof: Proof<PE>,
        amt_high_commitment: G1Curve,
        merkle_root: [Bytes32; COSET_N],
        merkle_proof: Vec<Bytes32>,
        merkle_leaf: Bytes32,
    ) -> Self {
        Self { index, amt_commitment, amt_proof, amt_high_commitment, merkle_root, merkle_proof, merkle_leaf }
    }
}

impl LightEncodedSlice {
    pub(crate) fn from_slice(slice: &EncodedSlice) -> Self {
        slice.check_amt_idx().unwrap();
        slice.check_merkle_idx().unwrap();
        
        let (amt_commitment, amt_proof, amt_high_commitment) = slice.amt_fields();
        let (merkle_root, merkle_proof, merkle_leaf) = slice.merkle_fields();

        Self::new(slice.index, amt_commitment, amt_proof, amt_high_commitment, merkle_root, merkle_proof, merkle_leaf)
    }
}