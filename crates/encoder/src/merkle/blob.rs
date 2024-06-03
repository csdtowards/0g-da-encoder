use super::{slice::EncodedSliceMerkle, Bytes32};
use crate::{
    constants::{BLOB_ROW_ENCODED, BLOB_ROW_LOG, COSET_N, ENCODED_BLOB_SIZE},
    utils::keccak_chunked,
};
use std::collections::VecDeque;

pub struct EncodedBlobMerkle {
    pub data: Vec<Bytes32>,
    pub tree: Vec<Vec<Bytes32>>,
}

impl EncodedBlobMerkle {
    #[tracing::instrument(skip_all, name = "encode_merkle", level = 2)]
    pub fn build(data: Vec<Bytes32>) -> Self {
        assert_eq!(data.len(), ENCODED_BLOB_SIZE);

        let leaves = keccak_chunked(&data, 8);

        let mut tree = VecDeque::new();
        let mut last_layer = leaves;
        while last_layer.len() > COSET_N {
            let next_layer = keccak_chunked(&last_layer, 2);
            let mut to_push_layer = next_layer;
            std::mem::swap(&mut last_layer, &mut to_push_layer);
            tree.push_front(to_push_layer);
        }
        tree.push_front(last_layer);

        Self {
            data,
            tree: tree.into(),
        }
    }

    pub fn root(&self) -> [Bytes32; COSET_N] {
        self.tree[0].clone().try_into().unwrap()
    }

    pub fn row_root(&self, index: usize) -> Bytes32 {
        self.tree[BLOB_ROW_LOG][index]
    }

    pub fn get_row(&self, index: usize) -> EncodedSliceMerkle {
        assert!(index < BLOB_ROW_ENCODED);

        let proof = (1..=BLOB_ROW_LOG)
            .rev()
            .map(|d| {
                let height = BLOB_ROW_LOG - d;
                let idx = index >> height;
                self.tree[d][idx ^ 1]
            })
            .collect();

        EncodedSliceMerkle::new(self.root(), proof, index, self.row_root(index))
    }
}

impl EncodedBlobMerkle {
    #[cfg(any(test, feature = "testonly_code"))]
    pub(crate) fn get_invalid_row(
        &self, index: usize, err_code: &ErrCodeMerkle,
    ) -> EncodedSliceMerkle {
        use ethereum_types::H256;

        assert!(index < BLOB_ROW_ENCODED);

        let mut proof: Vec<_> = (1..=BLOB_ROW_LOG)
            .rev()
            .map(|d| {
                let height = BLOB_ROW_LOG - d;
                let i = index >> height;
                self.tree[d][i ^ 1]
            })
            .collect();
        let mut leaf_index = index;
        let mut leaf = self.row_root(index);
        let mut root = self.root();
        match err_code {
            ErrCodeMerkle::WrongIndex => leaf_index += 1,
            ErrCodeMerkle::WrongLocalRoot => {
                leaf = H256::from_low_u64_be(H256(leaf).to_low_u64_be() + 1).0
            }
            ErrCodeMerkle::WrongProof => {
                proof[0] =
                    H256::from_low_u64_be(H256(proof[0]).to_low_u64_be() + 1).0
            }
            ErrCodeMerkle::WrongRoot => {
                root = root.map(|root| {
                    H256::from_low_u64_be(H256(root).to_low_u64_be() + 1).0
                })
            }
        }
        EncodedSliceMerkle::new(root, proof, leaf_index, leaf)
    }
}

#[cfg(any(test, feature = "testonly_code"))]
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum ErrCodeMerkle {
    WrongIndex,
    WrongLocalRoot,
    WrongProof,
    WrongRoot,
}

#[cfg(test)]
mod tests {
    use crate::{
        constants::{BLOB_COL_LOG, BLOB_ROW_LOG, COSET_N, ENCODED_BLOB_SIZE},
        merkle::blob::EncodedBlobMerkle,
    };

    #[test]
    fn test_merkle_build() {
        let EncodedBlobMerkle { tree, .. } = EncodedBlobMerkle::build(vec![
                Default::default();
                ENCODED_BLOB_SIZE
            ]);

        assert_eq!(tree.len(), BLOB_ROW_LOG + BLOB_COL_LOG - 2); // logrow+logcol+1-3 -> 0
        assert_eq!(
            tree[BLOB_ROW_LOG + BLOB_COL_LOG - 3].len(),
            ENCODED_BLOB_SIZE >> 3
        );
        assert_eq!(tree[0].len(), COSET_N);
    }
}
