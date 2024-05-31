pub mod fast_serde;
mod generate;
mod interfaces;
mod prove;
#[cfg(feature = "cuda")]
mod prove_gpu;
mod serde;
mod verify;

#[cfg(test)]
mod tests;

use crate::ec_algebra::{G1Aff, G2Aff, Pairing, G2};

pub use interfaces::AMTProofs;
#[cfg(feature = "cuda")]
use parking_lot::RwLock;

pub struct AMTParams<PE: Pairing> {
    pub basis: Vec<G1Aff<PE>>,
    pub quotients: Vec<Vec<G1Aff<PE>>>,
    pub vanishes: Vec<Vec<G2Aff<PE>>>,
    pub g2: G2<PE>,
    pub high_basis: Vec<G1Aff<PE>>,
    pub high_g2: G2<PE>,
    #[cfg(feature = "cuda")]
    device_mem: RwLock<Option<prove_gpu::MsmBasisOnDevice>>,
}

impl<PE: Pairing> AMTParams<PE> {
    pub fn new(
        basis: Vec<G1Aff<PE>>, quotients: Vec<Vec<G1Aff<PE>>>,
        vanishes: Vec<Vec<G2Aff<PE>>>, g2: G2<PE>,
        high_basis: Vec<G1Aff<PE>>, high_g2: G2<PE>,
    ) -> Self {
        Self {
            basis,
            quotients,
            vanishes,
            g2,
            high_basis,
            high_g2,
            #[cfg(feature = "cuda")]
            device_mem: RwLock::new(None),
        }
    }
}

impl<PE: Pairing> PartialEq for AMTParams<PE> {
    fn eq(&self, other: &Self) -> bool {
        self.basis == other.basis
            && self.quotients == other.quotients
            && self.vanishes == other.vanishes
            && self.g2 == other.g2
            && self.high_basis == other.high_basis
            && self.high_g2 == other.high_g2
    }
}

impl<PE: Pairing> Eq for AMTParams<PE> {}
