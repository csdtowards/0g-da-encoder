mod amt;
pub mod constants;
mod encoder;
mod merkle;
mod raw_blob;
mod raw_data;
mod utils;

pub use amt::{blob::EncodedBlobAMT, slice::EncodedSliceAMT};
pub use encoder::{
    blob::EncodedBlob, error::EncoderError, slice::EncodedSlice, light_slice::LightEncodedSlice,
};
pub use merkle::{blob::EncodedBlobMerkle, slice::EncodedSliceMerkle};
pub use raw_blob::RawBlob;
pub use raw_data::RawData;
pub use utils::scalar_to_h256;

pub type ZgEncoderParams = ::amt::EncoderParams<
    ark_bn254::Bn254,
    { constants::COSET_N },
    { constants::BLOB_COL_LOG },
    { constants::BLOB_ROW_LOG },
>;

pub type ZgSignerParams = ::amt::VerifierParams<
    ark_bn254::Bn254,
    { constants::COSET_N },
    { constants::BLOB_COL_LOG },
    { constants::BLOB_ROW_LOG },
>;
