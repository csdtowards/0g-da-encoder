use std::{fs::File, path::Path};

use super::AMTParams;
use crate::{
    ec_algebra::{
        k_adicity, CanonicalDeserialize, CanonicalSerialize, EvaluationDomain,
        Field, Fr, G1Aff, G2Aff, One, Pairing, Radix2EvaluationDomain, Zero,
        G1, G2,
    },
    error, fast_serde,
    power_tau::PowerTau,
    utils::{amtp_file_name, bitreverse, index_reverse},
};

use ark_bn254::Bn254;
use ark_ec::CurveGroup;
use ark_ff::FftField;
use ark_std::cfg_iter_mut;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use tracing::{debug, info, instrument, warn};

impl AMTParams<Bn254> {
    #[instrument(skip_all, name = "load_amt_params", level = 2, parent = None, fields(depth=expected_depth, coset=coset))]
    pub fn from_dir_mont(
        dir: impl AsRef<Path>, expected_depth: usize, create_mode: bool,
        coset: usize,
    ) -> Self {
        debug!(
            depth = expected_depth,
            coset, "Load AMT params (mont format)"
        );
        let file_name = amtp_file_name::<Bn254>(expected_depth, coset, true);
        let path = dir.as_ref().join(file_name);

        if let Ok(params) = Self::load_cached_mont(&path) {
            return params;
        }

        if !create_mode {
            panic!("Fail to load amt params in mont from {:?}", path);
        }

        info!("Fail to load AMT params (mont format)");

        // let params = Self::from_dir(dir, expected_depth, create_mode, coset);
        let pp = PowerTau::<Bn254>::from_dir_mont(dir, expected_depth, create_mode);
        let params = Self::from_pp(pp, coset);

        let writer = File::create(&*path).unwrap();

        info!(file = ?path, "Save generated AMT params (mont format)");
        fast_serde::write(&params, writer).unwrap();

        params
    }

    fn load_cached_mont(file: impl AsRef<Path>) -> Result<Self, error::Error> {
        let buffer = File::open(file)?;
        Ok(fast_serde::read(buffer)?)
    }
}

impl<PE: Pairing> AMTParams<PE> {
    #[instrument(skip_all, name = "load_amt_params", level = 2, parent = None, fields(depth=expected_depth, coset=coset))]
    pub fn from_dir(
        dir: impl AsRef<Path>, expected_depth: usize, create_mode: bool,
        coset: usize,
    ) -> Self {
        debug!(
            depth = expected_depth,
            coset, "Load AMT params (unmont format)"
        );

        let file_name = amtp_file_name::<PE>(expected_depth, coset, false);
        let path = dir.as_ref().join(file_name);

        if let Ok(params) = Self::load_cached(&path) {
            return params;
        }

        info!("Fail to load AMT params (unmont format)");

        if !create_mode {
            panic!("Fail to load amt params from {:?}", path);
        }

        info!("Construct a new AMT params");

        let pp = PowerTau::<PE>::from_dir(dir, expected_depth, create_mode);

        let params = Self::from_pp(pp, coset);
        let buffer = File::create(&path).unwrap();

        info!(file = ?path, "Save generated AMT params (unmont format)");
        params.serialize_uncompressed(&buffer).unwrap();

        params
    }

    fn load_cached(file: impl AsRef<Path>) -> Result<Self, error::Error> {
        let mut buffer = File::open(file)?;
        Ok(CanonicalDeserialize::deserialize_uncompressed_unchecked(
            &mut buffer,
        )?)
    }

    pub fn len(&self) -> usize { self.basis.len() }

    fn enact<T: CurveGroup>(input: Vec<T>) -> Vec<<T as CurveGroup>::Affine> {
        let mut affine = CurveGroup::normalize_batch(input.as_slice());
        index_reverse(&mut affine);
        affine
    }

    pub fn coset_factor(length: usize, idx: usize) -> Fr<PE> {
        assert!(length.is_power_of_two());
        let depth = ark_std::log2(length) as usize;
        let two_adicity: usize = <Fr<PE> as FftField>::TWO_ADICITY as usize;
        assert!(depth <= two_adicity);
        assert!(idx < 1 << (two_adicity - depth));
        let pow = bitreverse(idx, two_adicity - depth);

        <Fr<PE> as FftField>::TWO_ADIC_ROOT_OF_UNITY.pow(&[pow as u64])
    }

    pub fn from_pp(pp: PowerTau<PE>, coset: usize) -> Self {
        info!("Generate AMT params from powers of tau");

        let (mut g1pp, mut g2pp) = pp.into_projective();

        assert_eq!(g1pp.len(), g2pp.len());
        assert!(g1pp.len().is_power_of_two());
        let length = g1pp.len();
        let depth = ark_std::log2(length) as usize;

        if coset > 0 {
            debug!(coset, "Adjust powers of tau according to coset index");

            let w = Fr::<PE>::one() / Self::coset_factor(length, coset);
            cfg_iter_mut!(g1pp).enumerate().for_each(
                |(idx, x): (_, &mut G1<PE>)| *x *= w.pow([idx as u64]),
            );
            cfg_iter_mut!(g2pp).enumerate().for_each(
                |(idx, x): (_, &mut G2<PE>)| *x *= w.pow([idx as u64]),
            );
        }

        let fft_domain = Radix2EvaluationDomain::<Fr<PE>>::new(length).unwrap();

        let basis: Vec<G1Aff<PE>> =
            Self::enact(Self::gen_basis(&g1pp[..], &fft_domain));
        let quotients: Vec<Vec<G1Aff<PE>>> = (1..=depth)
            .map(|d| {
                Self::enact(Self::gen_quotients(&g1pp[..], &fft_domain, d))
            })
            .collect();
        let vanishes: Vec<Vec<G2Aff<PE>>> = (1..=depth)
            .map(|d| Self::enact(Self::gen_vanishes(&g2pp[..], d)))
            .collect();

        Self::new(basis, quotients, vanishes, g2pp[0])
    }

    fn gen_basis(
        g1pp: &[G1<PE>], fft_domain: &Radix2EvaluationDomain<Fr<PE>>,
    ) -> Vec<G1<PE>> {
        debug!("Generate basis");
        fft_domain.ifft(g1pp)
    }

    fn gen_quotients(
        g1pp: &[G1<PE>], fft_domain: &Radix2EvaluationDomain<Fr<PE>>,
        depth: usize,
    ) -> Vec<G1<PE>> {
        debug!(depth, "Generate quotients");

        assert!(g1pp.len() <= 1 << 32);

        let length = g1pp.len();
        let max_depth = k_adicity(2, length as u64) as usize;

        assert_eq!(1 << max_depth, length);
        assert!(max_depth >= depth);
        assert!(depth >= 1);

        let mut coeff = vec![G1::<PE>::zero(); length];
        let max_coeff = 1usize << (max_depth - depth);
        for i in 1..=max_coeff {
            coeff[i] = g1pp[max_coeff - i];
        }

        let mut answer = fft_domain.fft(&coeff);
        cfg_iter_mut!(answer, 1024)
            .for_each(|val: &mut G1<PE>| *val *= fft_domain.size_inv);
        answer
    }

    fn gen_vanishes(g2pp: &[G2<PE>], depth: usize) -> Vec<G2<PE>> {
        debug!(depth, "Generate vanishes");

        assert!(g2pp.len() <= 1 << 32);

        let length = g2pp.len();
        let max_depth = k_adicity(2, length as u64) as usize;

        assert_eq!(1 << max_depth, length);
        assert!(max_depth >= depth);
        assert!(depth >= 1);

        let height = max_depth - depth;
        let step = 1 << height;
        let mut fft_domain =
            Radix2EvaluationDomain::<Fr<PE>>::new(1 << depth).unwrap();

        let mut coeff = vec![G2::<PE>::zero(); 1 << depth];

        coeff[0] = g2pp[length - step];
        for i in 1..length / step {
            coeff[i] = g2pp[(i - 1) * step]
        }

        std::mem::swap(
            &mut fft_domain.group_gen,
            &mut fft_domain.group_gen_inv,
        );
        fft_domain.fft(&coeff)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{
        TestParams, DOMAIN, G1PP, G2PP, PE, PP, TEST_LENGTH, TEST_LEVEL, W,
    };
    use crate::ec_algebra::{
        EvaluationDomain, Field, Fr, One, Pairing, VariableBaseMSM, Zero, G1,
        G2,
    };

    fn simple_gen_basis(index: usize) -> G1<PE> {
        let mut points = vec![Fr::<PE>::zero(); TEST_LENGTH];
        points[index] = Fr::<PE>::one();

        let coeff = DOMAIN.ifft(&points);
        G1::<PE>::msm(&PP.0, &coeff[..]).unwrap()
    }

    #[test]
    fn test_gen_basis() {
        let indents = TestParams::gen_basis(&G1PP, &*DOMAIN);
        for t in 0..TEST_LENGTH {
            assert_eq!(indents[t], simple_gen_basis(t))
        }
    }

    fn simple_gen_quotinents(index: usize, depth: usize) -> G1<PE> {
        let size = TEST_LENGTH / (1 << depth);
        (0..size)
            .rev()
            .map(|j| W.pow(&[(index * j) as u64]))
            .zip(PP.0[0..size].iter())
            .map(|(exp, base)| *base * exp)
            .sum::<G1<PE>>()
            * DOMAIN.size_inv
            * W.pow(&[index as u64])
    }

    #[test]
    fn test_gen_quotients() {
        for depth in (1..=TEST_LEVEL).rev() {
            let quotients = TestParams::gen_quotients(&G1PP, &DOMAIN, depth);
            for t in 0..TEST_LENGTH {
                assert_eq!(quotients[t], simple_gen_quotinents(t, depth));
            }
        }
    }

    fn simple_gen_vanishes(index: usize, depth: usize) -> G2<PE> {
        let step = TEST_LENGTH / (1 << depth);
        let size = 1 << depth;
        (0..size)
            .rev()
            .map(|j| W.pow(&[(index * step * j) as u64]))
            .zip(PP.1.iter().step_by(step))
            .map(|(exp, base)| *base * exp)
            .sum()
    }

    #[test]
    fn test_gen_vanishes() {
        for depth in (1..=TEST_LEVEL).rev() {
            let vanishes = TestParams::gen_vanishes(&G2PP, depth);
            for t in 0..TEST_LENGTH {
                assert_eq!(
                    vanishes[t % (1 << depth)],
                    simple_gen_vanishes(t, depth)
                );
            }
        }
    }

    #[test]
    fn test_simple_gen_params() {
        for depth in (1..=TEST_LEVEL).rev() {
            for t in 0..TEST_LENGTH {
                assert_eq!(
                    PE::pairing(simple_gen_basis(t), G2PP[0]),
                    PE::pairing(
                        simple_gen_quotinents(t, depth),
                        simple_gen_vanishes(t, depth)
                    )
                );
            }
        }
    }

    #[test]
    fn test_gen_params() {
        let basis = TestParams::gen_basis(&G1PP, &DOMAIN);
        for depth in (1..=TEST_LEVEL).rev() {
            let prove_data = TestParams::gen_quotients(&G1PP, &DOMAIN, depth);
            let verify_data = TestParams::gen_vanishes(&G2PP, depth);
            for t in 0..TEST_LENGTH {
                assert_eq!(
                    PE::pairing(basis[t], G2PP[0]),
                    PE::pairing(prove_data[t], verify_data[t % (1 << depth)])
                );
            }
        }
    }
}
