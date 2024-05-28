use crate::{
    ec_algebra::{
        AffineRepr, CanonicalDeserialize, CanonicalSerialize, CurveGroup, Fr,
        G1Aff, G2Aff, Pairing, UniformRand, G1, G2,
    },
    error, pp_file_name,
};
use ark_ff::{utils::k_adicity, Field};
use ark_std::cfg_into_iter;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::{
    fs::{create_dir_all, File},
    path::Path,
};
use tracing::{debug, info};

#[derive(CanonicalDeserialize, CanonicalSerialize, Clone)]
pub struct PowerTau<PE: Pairing>(pub Vec<G1Aff<PE>>, pub Vec<G2Aff<PE>>);

fn power_tau<'a, G: AffineRepr>(
    gen: &'a G, tau: &'a G::ScalarField, length: usize,
) -> Vec<G> {
    let gen: G::Group = gen.into_group();
    cfg_into_iter!(0usize..length)
        .step_by(1024)
        .flat_map(|x| {
            let project_tau: Vec<_> = (x..std::cmp::min(x + 1024, length))
                .map(|idx| gen * tau.pow([idx as u64]))
                .collect();
            CurveGroup::normalize_batch(&project_tau[..])
        })
        .collect()
}

impl<PE: Pairing> PowerTau<PE> {
    #[cfg(test)]
    fn setup_with_tau(tau: Fr<PE>, depth: usize) -> PowerTau<PE> {
        Self::setup_inner(Some(tau), depth)
    }

    pub fn setup(depth: usize) -> PowerTau<PE> {
        Self::setup_inner(None, depth)
    }

    fn setup_inner(tau: Option<Fr<PE>>, depth: usize) -> PowerTau<PE> {
        info!(random_tau = tau.is_none(), depth, "Setup powers of tau");

        let random_tau = Fr::<PE>::rand(&mut rand::thread_rng());
        let tau = tau.unwrap_or(random_tau);

        let gen1 = G1Aff::<PE>::generator();
        let gen2 = G2Aff::<PE>::generator();

        let g1pp: Vec<G1Aff<PE>> = power_tau(&gen1, &tau, 1 << depth);
        let g2pp: Vec<G2Aff<PE>> = power_tau(&gen2, &tau, 1 << depth);

        PowerTau(g1pp, g2pp)
    }

    fn from_dir_inner(
        file: impl AsRef<Path>, expected_depth: usize,
    ) -> Result<PowerTau<PE>, error::Error> {
        let buffer = File::open(file)?;
        let pp: PowerTau<PE> =
            CanonicalDeserialize::deserialize_compressed_unchecked(buffer)?;

        let (g1_len, g2_len) = (pp.0.len(), pp.1.len());
        let depth = k_adicity(2, g1_len as u64) as usize;

        if g1_len != g2_len || expected_depth > depth {
            Err(error::ErrorKind::InconsistentLength.into())
        } else if expected_depth < g2_len {
            let g1_vec = pp.0[..1 << expected_depth].to_vec();
            let g2_vec = pp.1[..1 << expected_depth].to_vec();
            Ok(PowerTau(g1_vec, g2_vec))
        } else {
            Ok(pp)
        }
    }

    pub fn from_dir(
        dir: impl AsRef<Path>, expected_depth: usize, create_mode: bool,
    ) -> PowerTau<PE> {
        debug!("Load powers of tau");

        let file = &dir.as_ref().join(pp_file_name::<PE>(expected_depth));
        if let Ok(loaded) = Self::from_dir_inner(file, expected_depth) {
            return loaded;
        }
        info!("Fail to load powers of tau");

        if !create_mode {
            panic!(
                "Fail to load public parameters for {} at depth {}, read TODO to generate",
                std::any::type_name::<PE>(),
                expected_depth
            );
        }

        let pp = Self::setup(expected_depth);
        create_dir_all(Path::new(file).parent().unwrap()).unwrap();
        let buffer = File::create(file).unwrap();
        info!(?file, "Save generated powers of tau");
        pp.serialize_compressed(&buffer).unwrap();
        pp
    }

    pub fn into_projective(self) -> (Vec<G1<PE>>, Vec<G2<PE>>) {
        let g1pp = self.0.into_iter().map(G1::<PE>::from).collect();
        let g2pp = self.1.into_iter().map(G2::<PE>::from).collect();
        (g1pp, g2pp)
    }
}

#[test]
fn test_partial_load() {
    type PE = ark_bn254::Bn254;

    let tau = Fr::<PE>::rand(&mut rand::thread_rng());
    let large_pp = PowerTau::<PE>::setup_with_tau(tau, 8);
    let small_pp = PowerTau::<PE>::setup_with_tau(tau, 4);

    assert_eq!(small_pp.0[..], large_pp.0[..(small_pp.0.len())]);
    assert_eq!(small_pp.1[..], large_pp.1[..(small_pp.1.len())]);
}

#[test]
fn test_parallel_build() {
    use crate::ec_algebra::CurveGroup;

    const DEPTH: usize = 13;
    type PE = ark_bn254::Bn254;

    let tau = Fr::<PE>::rand(&mut rand::thread_rng());
    let gen1 = G1Aff::<PE>::generator();
    let g1pp_ans = power_tau(&gen1, &tau, 1 << DEPTH);

    let mut g1pp: Vec<G1Aff<PE>> = vec![];
    g1pp.reserve(1 << DEPTH);
    let mut gen1 = gen1.into_group();
    for _ in 0..1 << DEPTH {
        g1pp.push(gen1.into_affine());
        gen1 *= tau.clone();
    }
    assert_eq!(g1pp, g1pp_ans)
}
