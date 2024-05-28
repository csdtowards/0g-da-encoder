use amt::AmtProofError;

#[derive(Debug, PartialEq, Eq)]
pub enum AmtError {
    IncorrectCommitment,
    IncorrectRowSize {
        actual: usize,
        expected: usize,
    },
    RowIndexOverflow {
        actual: usize,
        expected_max: usize,
    },
    // slice.index = coset_index * num_cosets + local_index,
    // amt_index should equal to local_index
    UnmatchedCosetIndex {
        coset_index: usize,
        local_index: usize,
        amt_index: usize,
    },
    IncorrectProof {
        coset_index: usize,
        amt_index: usize,
        error: AmtProofError,
    },
}
