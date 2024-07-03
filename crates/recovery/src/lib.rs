#[macro_use]
extern crate tracing;

mod data_poly;
mod data_times_zpoly;
mod error;
mod poly;
mod utils;
mod zpoly;

use std::collections::BTreeMap;

use zg_encoder::RawData;

pub fn recover_from_da_slice(
    slices: &BTreeMap<usize, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    use data_poly::data_poly;
    use utils::raw_slice_to_line;
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
