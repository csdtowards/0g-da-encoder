use crate::ec_algebra::{
        G1Aff, G2, Pairing, 
    };

pub struct LDTParams<PE: Pairing> {
    pub g1s_ifft: Vec<G1Aff<PE>>, // let fft_domain = Radix2EvaluationDomain::<Fr<PE>>::new(1 << depth).unwrap();
    pub g2: G2<PE>
}

pub struct LDTVerifyParams<PE: Pairing>(pub G2<PE>);