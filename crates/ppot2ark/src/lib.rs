#![allow(dead_code, unused)]

mod adapter;

pub use adapter::Adapter;

use amt::{fast_serde::write_power_tau, ptau_file_name};
pub use ark_ec::pairing::Pairing;
pub use bellman_ce::pairing::bn256::Bn256;
pub use powersoftau::{
    batched_accumulator::BatchedAccumulator,
    parameters::{CeremonyParams, CheckForCorrectness, UseCompression},
};

use memmap::MmapOptions;
use std::{
    fs::{self, read, File, OpenOptions},
    path::Path,
};
use zg_encoder::constants::{BLOB_COL_LOG, BLOB_ROW_LOG};

use ark_bn254::{Bn254, G1Affine, G2Affine};
pub struct PowerTauLight(pub Vec<G1Affine>, pub Vec<G2Affine>);
type PowerTau = amt::PowerTau<Bn254>;
use project_root;

#[derive(Debug, Clone, Copy)]
pub enum InputType {
    Challenge,
    Response,
}

impl InputType {
    fn file_name(&self, degree: usize) -> String {
        format!(
            "{}_{}",
            match self {
                InputType::Challenge => "challenge",
                InputType::Response => "response",
            },
            degree
        )
    }
}

fn from_ppot_file_inner<'a>(
    input_path: &str, input_type: InputType, file_size: usize,
    read_from: usize, read_size_pow: usize, chunk_size_pow: usize,
    parameters: &'a CeremonyParams<Bn256>,
) -> Result<PowerTauLight, String> {
    // let read_from = (1 << read_from)
    // - 1;
    let read_size = 1 << read_size_pow;
    let chunk_size = 1 << chunk_size_pow;

    if (read_from + read_size) > (1 << file_size) {
        return Err(format!("too long to read"));
    }

    let input_filename =
        format!("{}/{}", input_path, input_type.file_name(file_size));

    let reader = OpenOptions::new()
        .read(true)
        .open(&input_filename)
        .map_err(|e| format!("Cannot open {}: {:?}", input_filename, e))?;

    let input_map = unsafe {
        MmapOptions::new().map(&reader).map_err(|e| {
            format!("unable to create a memory map for input, detail: {}", e)
        })?
    };

    let mut accumulator = BatchedAccumulator::empty(parameters);
    let use_compression = if let InputType::Response = input_type {
        UseCompression::Yes
    } else {
        UseCompression::No
    };

    let mut g1 = Vec::with_capacity(read_size);
    let mut g2 = Vec::with_capacity(read_size);

    let mut read_offset = read_from;
    let mut remaining_size = read_size;
    while remaining_size > 0 {
        let current_chunk_size = std::cmp::min(chunk_size, remaining_size);
        accumulator
            .read_chunk(
                read_offset,
                current_chunk_size,
                use_compression,
                CheckForCorrectness::Yes,
                &input_map,
            )
            .map_err(|e| format!("failed to read chunk, detail: {}", e))?;

        g1.extend(
            accumulator.tau_powers_g1[..current_chunk_size]
                .iter()
                .map(|tau| tau.adapt()),
        );
        g2.extend(
            accumulator.tau_powers_g2[..current_chunk_size]
                .iter()
                .map(|tau| tau.adapt()),
        );

        read_offset += current_chunk_size;
        remaining_size -= current_chunk_size;
    }

    Ok(PowerTauLight(g1, g2))
}

pub fn from_ppot_file(
    input_path: &str, input_type: InputType, file_size_pow: usize,
    read_from: usize, read_size_pow: usize, chunk_size_pow: usize,
) -> Result<PowerTauLight, String> {
    let params = CeremonyParams::<Bn256>::new(file_size_pow, file_size_pow);
    from_ppot_file_inner(
        input_path,
        input_type,
        file_size_pow,
        read_from,
        read_size_pow,
        chunk_size_pow,
        &params,
    )
}

pub fn from_ppot_file_ldt(
    input_path: &str, input_type: InputType, file_size_pow: usize,
    read_size_pow: usize, high_read_size_pow: usize, chunk_size_pow: usize,
) -> Result<PowerTau, String> {
    let PowerTauLight(g1pp, g2pp) = from_ppot_file(
        input_path,
        input_type,
        file_size_pow,
        0,
        read_size_pow,
        chunk_size_pow,
    )?;
    let high_read_from = (1 << high_read_size_pow) - (1 << read_size_pow);
    let PowerTauLight(high_g1pp, high_g2pp) = from_ppot_file(
        input_path,
        input_type,
        file_size_pow,
        high_read_from,
        read_size_pow,
        chunk_size_pow,
    )?;
    Ok(amt::PowerTau {
        g1pp,
        g2pp,
        high_g1pp,
        high_g2: high_g2pp[0].into(),
    })
}

pub fn load_save_power_tau(
    input_path: &str, input_type: InputType, file_size_pow: usize,
    read_size_pow: usize, high_read_size_pow: usize, chunk_size_pow: usize,
    dir: impl AsRef<Path>,
) -> Result<(), String> {
    let power_tau = from_ppot_file_ldt(
        input_path,
        input_type,
        file_size_pow,
        read_size_pow,
        high_read_size_pow,
        chunk_size_pow,
    )?;
    let path = dir.as_ref().join(ptau_file_name::<Bn254>(
        read_size_pow,
        true,
    ));
    let writer = File::create(&*path).unwrap();
    write_power_tau(&power_tau, writer).unwrap();
    Ok(())
}

fn main() {
    let input_path = format!("{}/data", crate_path());
    let input_type = InputType::Challenge;
    let file_size_pow = 12;
    let read_size_pow = BLOB_COL_LOG + BLOB_ROW_LOG;
    let high_read_size_pow = 28;
    let chunk_size_pow = 10;
    let dir = "../pp";
    let pot = load_save_power_tau(
        &input_path,
        input_type,
        file_size_pow,
        read_size_pow,
        high_read_size_pow,
        chunk_size_pow,
        dir,
    )
    .unwrap();
}

fn crate_path() -> String {
    let mut p = project_root::get_project_root().unwrap();
    p.push("crates/ppot2ark");
    p.to_str().unwrap().into()
}

#[cfg(test)]
mod tests {
    use std::{fs::read, path::PathBuf, process::Command};

    use super::*;

    fn data_path() -> String { format!("{}/data", crate_path()) }

    fn prepare_test_file(ty: InputType, degree: usize) {
        let target_file = format!("{}/{}", data_path(), ty.file_name(degree));
        let script = format!("{}/gen_test_ppot.sh", crate_path());

        if std::fs::metadata(target_file.clone()).is_ok() {
            return;
        }

        println!("{} not found, building...", target_file);

        Command::new("bash")
            .arg(script)
            .arg(degree.to_string())
            .output()
            .expect("Failed to execute command");
    }

    #[test]
    fn test_from_ppot_file_ldt() {
        let input_path = format!("{}/data", crate_path());
        let input_type = InputType::Challenge;
        let file_size_pow = 12;
        let read_size_pow = 8;
        let high_read_size_pow = 11;
        let chunk_size_pow = 10;

        prepare_test_file(input_type, file_size_pow);
        let pot = from_ppot_file_ldt(
            &input_path,
            input_type,
            file_size_pow,
            read_size_pow,
            high_read_size_pow,
            chunk_size_pow,
        )
        .unwrap();
        pot.check_ldt();
    }

    #[test]
    fn test_load_from_challenge_12_nomal() {
        let input_path = format!("{}/data", crate_path());
        let input_type = InputType::Challenge;
        let file_size_pow = 12;
        let read_from = 3840;
        let read_size_pow = 8;
        let chunk_size_pow = 10;

        prepare_test_file(input_type, file_size_pow);
        let pot = from_ppot_file(
            &input_path,
            input_type,
            file_size_pow,
            read_from,
            read_size_pow,
            chunk_size_pow,
        )
        .unwrap();
        assert_eq!(pot.0.len(), 1 << read_size_pow);
        assert_eq!(
            Bn254::pairing(pot.0[0], pot.1[4]),
            Bn254::pairing(pot.0[1], pot.1[3])
        );
    }

    #[test]
    fn test_load_from_challenge_12_too_long() {
        let input_path = format!("{}/data", crate_path());
        let input_type = InputType::Challenge;
        let file_size_pow = 12;
        let read_from = 3841;
        let read_size_pow = 8;
        let chunk_size_pow = 10;

        prepare_test_file(input_type, file_size_pow);
        let pot = from_ppot_file(
            &input_path,
            input_type,
            file_size_pow,
            read_from,
            read_size_pow,
            chunk_size_pow,
        );
        assert!(matches!(pot, Err(ref msg) if msg == "too long to read"));
    }

    //#[ignore = "heavy task"]
    #[test]
    fn test_load_from_high_deg_response_nomal() {
        // expect to deg 28
        let input_path = format!("{}/data", crate_path());
        let input_type = InputType::Response;
        let file_size_pow = 12;
        let read_size_pow = 8;
        let chunk_size_pow = 10;
        let read_from = 2u32.pow(file_size_pow) - 2u32.pow(read_size_pow);

        prepare_test_file(input_type, file_size_pow as usize);
        let pot = from_ppot_file(
            &input_path,
            input_type,
            file_size_pow as usize,
            read_from as usize,
            read_size_pow as usize,
            chunk_size_pow,
        )
        .unwrap();
        println!("powers length: {}", pot.0.len());
        assert_eq!(pot.0.len(), 1 << read_size_pow);
        assert_eq!(
            Bn254::pairing(pot.0[0], pot.1[4]),
            Bn254::pairing(pot.0[1], pot.1[3])
        );
    }
}
