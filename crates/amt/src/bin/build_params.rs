use amt::{AMTParams, AMTVerifyParams, PowerTau};
use anyhow::{bail, Result};
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

fn parse_param() -> Result<(usize, usize, usize, Option<String>)> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        bail!(
            "Usage: {} <amt-depth> <verify-depth> <coset-index> [<power_tau_dir>]",
            args[0]
        );
    }

    let path = if args.len() == 5 {
        Some(args[4].parse()?)
    } else {
        None
    };

    Ok((args[1].parse()?, args[2].parse()?, args[3].parse()?, path))
}

fn main() {
    let (expected_depth, verify_depth, coset, ptau_dir) = match parse_param() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Cannot parse input: {:?}", e);
            std::process::exit(1);
        }
    };

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(false)
        .init();

    let pp = if let Some(dir) = ptau_dir {
        Some(PowerTau::from_dir(dir, expected_depth, false))
    } else {
        None
    };

    AMTParams::from_dir_mont("./pp", expected_depth, coset, true, pp.as_ref());
    AMTVerifyParams::from_dir_mont("./pp", expected_depth, verify_depth, coset);
}
