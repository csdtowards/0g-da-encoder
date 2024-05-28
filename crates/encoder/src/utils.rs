use crate::constants::Scalar;
use ark_ff::{BigInt, MontConfig};
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use tiny_keccak::{Hasher, Keccak};

pub fn raw_unit_to_scalar(chunk: &[u8]) -> Scalar {
    let mut raw: [u8; 32] = [0u8; 32];
    raw[..31].copy_from_slice(chunk);
    let big_int: BigInt<4> = BigInt(unsafe { std::mem::transmute(raw) });
    MontConfig::from_bigint(big_int).unwrap()
}

pub fn scalar_to_h256(scalar: Scalar) -> [u8; 32] {
    let bytes: [u64; 4] = MontConfig::into_bigint(scalar).0;
    let raw = unsafe { std::mem::transmute::<_, [u8; 32]>(bytes) };
    raw
}

#[macro_export]
macro_rules! cfg_chunks_exact {
    ($e: expr, $size: expr, $min_len: expr) => {{
        #[cfg(feature = "parallel")]
        let result = $e.par_chunks_exact($size).with_min_len($min_len);

        #[cfg(not(feature = "parallel"))]
        let result = $e.chunks_exact($size);

        result
    }};
    ($e: expr, $size: expr) => {{
        #[cfg(feature = "parallel")]
        let result = $e.par_chunks_exact($size);

        #[cfg(not(feature = "parallel"))]
        let result = $e.chunks_exact($size);

        result
    }};
}

type Bytes32 = [u8; 32];
pub fn keccak_chunked(input: &[Bytes32], chunk_size: usize) -> Vec<Bytes32> {
    cfg_chunks_exact!(input, chunk_size, 64 / chunk_size)
        .map(|x| {
            let mut result = Bytes32::default();
            let mut keccak256 = Keccak::v256();
            for s in x {
                keccak256.update(s.as_ref());
            }
            keccak256.finalize(&mut result);
            result
        })
        .collect()
}

pub fn keccak_tuple(x: Bytes32, y: Bytes32) -> Bytes32 {
    let mut keccak256 = Keccak::v256();
    keccak256.update(x.as_ref());
    keccak256.update(y.as_ref());
    let mut result = Bytes32::default();
    keccak256.finalize(&mut result);
    result
}

#[cfg(test)]
mod tests {
    use super::{raw_unit_to_scalar, scalar_to_h256};
    use crate::constants::Scalar;
    use ark_ff::{BigInteger, MontConfig};

    #[test]
    fn test_bytes_to_scalar() {
        let chunk = [0u8; 31];
        let scalar = raw_unit_to_scalar(&chunk);
        assert!(MontConfig::into_bigint(scalar).is_zero());
    }

    #[test]
    fn test_scalar_to_h256() {
        use ark_std::One;
        let one = Scalar::one();
        let one_h256 = scalar_to_h256(one);
        let mut one_gt = [0u8; 32];
        one_gt[0] = 1;
        assert_eq!(one_h256.as_ref(), one_gt);
    }
}
