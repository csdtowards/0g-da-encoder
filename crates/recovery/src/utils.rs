#[cfg(feature = "parallel")]
use rayon::prelude::*;

use std::cmp::max;

use amt::{bitreverse, change_matrix_direction, AMTParams};
use ark_ff::{Field, Zero};
use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};
use ark_std::{cfg_chunks_mut, log2};
#[cfg(test)]
use ark_std::{
    rand::{seq::SliceRandom, Rng},
    UniformRand,
};
use zg_encoder::{
    constants::{
        Scalar, BLOB_COL_LOG, BLOB_ROW_LOG, MAX_BLOB_SIZE, PE, RAW_BLOB_SIZE,
    },
    raw_unit_to_scalar,
};

use crate::{data_poly, poly::Poly, zpoly::COSET_MORE};

use std::collections::BTreeMap;
use zg_encoder::RawData;

pub fn recover_from_da_slice(
    slices: &BTreeMap<usize, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    use data_poly::data_poly;
    use raw_slice_to_line;
    use zg_encoder::constants::{Scalar, BLOB_ROW_N};

    let converted_lines: BTreeMap<usize, Vec<Scalar>> = slices
        .iter()
        .filter_map(|(idx, raw)| {
            Some((*idx, raw_slice_to_line(raw.as_slice()).ok()?))
        })
        .collect();

    let dropped = slices.len() - converted_lines.len();
    if dropped > 0 {
        info!("{:?} lines dropped because of incorrect format", dropped);
    }

    if converted_lines.len() < BLOB_ROW_N {
        return Err("Not enough valid lines".to_string());
    }

    let raw_blob = data_poly(&converted_lines)
        .map_err(|e| format!("Cannot recover data: {:?}", e))?;

    let raw_data: RawData = raw_blob.try_into()?;
    Ok(raw_data.as_bytes().to_vec())
}

pub fn raw_slice_to_line(slice: &[u8]) -> Result<Vec<Scalar>, String> {
    if slice.len() != MAX_BLOB_SIZE {
        return Err("Incorrect raw slice length".to_string());
    }
    slice
        .chunks_exact(32)
        .map(|x| {
            assert_eq!(x.len(), 32);
            if x[31] != 0 {
                return Err("A cell has more than 248 bits data".to_string());
            }

            Ok(raw_unit_to_scalar(x))
        })
        .collect()
}

const SPARSE_THRES: usize = 100;
pub fn many_non_zeros(vec: &[Scalar]) -> bool {
    vec.iter()
        .filter(|x| x != &&Scalar::zero())
        .nth(SPARSE_THRES - 1)
        .is_some()
}

#[cfg(test)]
pub fn random_scalars<R: Rng>(length: usize, rng: &mut R) -> Vec<Scalar> {
    (0..length).map(|_| Scalar::rand(rng)).collect::<Vec<_>>()
}

#[cfg(test)]
pub fn random_row_ids<R: Rng>(row_num: usize, rng: &mut R) -> Vec<usize> {
    use zg_encoder::constants::BLOB_ROW_ENCODED;

    let mut row_ids: Vec<usize> = (0..BLOB_ROW_ENCODED).collect();
    row_ids.shuffle(rng);
    row_ids[..row_num].to_vec()
}

pub fn fx_to_fkx(coeffs_fx: &[Scalar], k: Scalar) -> Vec<Scalar> {
    let mut coeffs = coeffs_fx.to_vec();
    cfg_chunks_mut!(coeffs, 16)
        .enumerate()
        .for_each(|(idx, chunks)| {
            let mut base = k.pow([idx as u64 * 16]);
            for x in chunks.iter_mut() {
                *x *= base;
                base *= k;
            }
        });
    coeffs
}

fn coeffs_to_evals_coset(coeffs: &[Scalar], coset_idx: usize) -> Vec<Scalar> {
    let coeffs = {
        if coset_idx != 0 {
            let coset_w =
                AMTParams::<PE>::coset_factor(RAW_BLOB_SIZE, coset_idx);
            fx_to_fkx(coeffs, coset_w)
        } else {
            coeffs.to_vec()
        }
    };

    assert!(coeffs.len() <= COSET_MORE * RAW_BLOB_SIZE + 1);
    assert!(RAW_BLOB_SIZE.is_power_of_two());
    let fft_degree = max(coeffs.len().next_power_of_two(), RAW_BLOB_SIZE);
    let fft_domain = Radix2EvaluationDomain::<Scalar>::new(fft_degree).unwrap();
    let mut evals = fft_domain.fft(&coeffs);
    if fft_degree > RAW_BLOB_SIZE {
        evals = evals
            .into_iter()
            .step_by(fft_degree / RAW_BLOB_SIZE)
            .collect();
    }
    change_matrix_direction(&mut evals, BLOB_ROW_LOG, BLOB_COL_LOG);
    evals
}

pub fn coeffs_to_evals_larger(coeffs: &[Scalar]) -> Vec<Scalar> {
    (0..COSET_MORE)
        .flat_map(|coset_idx| coeffs_to_evals_coset(coeffs, coset_idx))
        .collect()
}

pub fn coeffs_to_evals(coeffs: &[Scalar]) -> Vec<Scalar> {
    coeffs_to_evals_coset(coeffs, 0)
}

pub fn evals_to_poly(evals: &[Scalar]) -> Poly {
    assert_eq!(evals.len(), COSET_MORE * RAW_BLOB_SIZE);
    assert!(evals.len().is_power_of_two());

    let mut chunk_evals: Vec<Vec<_>> = evals
        .chunks_exact(RAW_BLOB_SIZE)
        .map(|chunk| chunk.to_vec())
        .collect();
    for chunk in chunk_evals.iter_mut() {
        change_matrix_direction(chunk, BLOB_COL_LOG, BLOB_ROW_LOG);
    }

    let bit_reverse = log2(COSET_MORE) as usize;
    let coset_indices: Vec<usize> = (0..COSET_MORE)
        .map(|x| bitreverse(x, bit_reverse))
        .collect();
    let mut transpose_evals: Vec<_> = coset_indices
        .into_iter()
        .flat_map(|x| chunk_evals[x].clone())
        .collect();

    change_matrix_direction(
        &mut transpose_evals,
        log2(RAW_BLOB_SIZE) as usize,
        log2(COSET_MORE) as usize,
    );

    let fft_domain =
        Radix2EvaluationDomain::<Scalar>::new(transpose_evals.len()).unwrap();
    let coeffs = fft_domain.ifft(&transpose_evals);
    Poly::from_vec(coeffs)
}
