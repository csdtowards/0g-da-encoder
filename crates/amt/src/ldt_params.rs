use crate::ec_algebra::{
        G1Aff, G2, Pairing, 
    };

pub struct LDTParams<PE: Pairing> {
    pub g1s: Vec<G1Aff<PE>>,
    pub g2: G2<PE>
}