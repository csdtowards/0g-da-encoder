#[derive(Debug, PartialEq, Eq)]
pub enum MerkleError {
    IncorrectRoot,
    IncorrectSize { actual: usize, expected: usize },
    RowIndexOverflow { actual: usize, expected_max: usize },
    IncorrectLocalRoot { row_index: usize },
    IncorrectProof { row_index: usize },
}
