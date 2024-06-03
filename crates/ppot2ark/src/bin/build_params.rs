use anyhow::{bail, Result};
use ppot2ark::{load_save_power_tau, InputType};
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

fn parse_param() -> Result<(usize, usize, usize)> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        bail!(
            "Usage: {} <file_size_pow> <read_size_pow> <high_read_size_pow>",
            args[0]
        );
    }

    let file_size_pow = args[1].parse()?;
    let read_size_pow = args[2].parse()?;
    let high_read_size_pow = args[3].parse()?;

    if file_size_pow < read_size_pow || file_size_pow < high_read_size_pow || read_size_pow > high_read_size_pow {
        bail!(
            "Usage: {} <file_size_pow> <read_size_pow> <high_read_size_pow>\n
            <file_size_pow> should be the largest, 
            <read_size_pow> should be the smallest",
            args[0]
        );
    }
    Ok((
        file_size_pow,
        read_size_pow,
        high_read_size_pow,
    ))
}

fn crate_path() -> String {
    let mut p = project_root::get_project_root().unwrap();
    p.push("crates/ppot2ark");
    p.to_str().unwrap().into()
}

fn main() {
    let (file_size_pow, read_size_pow, high_read_size_pow) =
        match parse_param() {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Cannot parse input: {:?}", e);
                std::process::exit(1);
            }
        };
    let input_path = format!("{}/data", crate_path());
    let input_type = InputType::Challenge;
    let chunk_size_pow = 10;
    let dir = "pp";
    load_save_power_tau(
        &input_path,
        input_type,
        file_size_pow,
        read_size_pow,
        high_read_size_pow,
        chunk_size_pow,
        dir,
    )
    .unwrap();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(false)
        .init();
}
