use crate::{
    ec_algebra::{Fr, Pairing, G1},
    proofs::AllProofs,
    AMTParams,
};

pub trait AMTProofs {
    type PE: Pairing;

    fn gen_amt_proofs(
        &self, ri_data: &[Fr<Self::PE>],
    ) -> (G1<Self::PE>, AllProofs<Self::PE>);

    fn warmup(&self) {}
}

#[cfg(not(feature = "cuda"))]
impl<PE: Pairing> AMTProofs for AMTParams<PE> {
    type PE = PE;

    fn gen_amt_proofs(
        &self, ri_data: &[Fr<Self::PE>],
    ) -> (G1<Self::PE>, AllProofs<Self::PE>) {
        self.gen_all_proofs(ri_data)
    }
}

#[cfg(feature = "cuda")]
use ag_cuda_ec::pairing_suite::PE;

#[cfg(feature = "cuda")]
impl AMTProofs for AMTParams<PE> {
    type PE = PE;

    fn gen_amt_proofs(
        &self, ri_data: &[Fr<Self::PE>],
    ) -> (G1<Self::PE>, AllProofs<Self::PE>) {
        self.gen_all_proofs_gpu(ri_data)
    }

    fn warmup(&self) {
        let _loaded = self.read_gpu_bases();
        ag_cuda_ec::init_local_workspace();
    }
}
