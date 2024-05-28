use crate::{
    ec_algebra::{Fr, Pairing, G1},
    proofs::AllProofs,
    AMTParams,
};

pub trait AMTProofs {
    type PE: Pairing;

    fn gen_amt_proofs(
        &self, ri_data: &[Fr<Self::PE>], batch_size: usize,
    ) -> (G1<Self::PE>, AllProofs<Self::PE>);

    fn warmup(&self, _height: usize) {}
}

#[cfg(not(feature = "cuda"))]
impl<PE: Pairing> AMTProofs for AMTParams<PE> {
    type PE = PE;

    fn gen_amt_proofs(
        &self, ri_data: &[Fr<Self::PE>], batch_size: usize,
    ) -> (G1<Self::PE>, AllProofs<Self::PE>) {
        self.gen_all_proofs(ri_data, batch_size)
    }
}

#[cfg(feature = "cuda")]
use ag_cuda_ec::pairing_suite::PE;

#[cfg(feature = "cuda")]
impl AMTProofs for AMTParams<PE> {
    type PE = PE;

    fn gen_amt_proofs(
        &self, ri_data: &[Fr<Self::PE>], batch_size: usize,
    ) -> (G1<Self::PE>, AllProofs<Self::PE>) {
        self.gen_all_proofs_gpu(ri_data, batch_size)
    }

    fn warmup(&self, height: usize) {
        let _ = self.read_gpu_bases(height);
        ag_cuda_ec::init_local_workspace();
    }
}
