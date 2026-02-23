use std::{env, fs, process, path::Path};

use btc_cli::models::Fixture;
use btc_cli::analyzer::analyze;
use btc_cli::block;

fn print_error(code: &str, message: &str) -> ! {
    let error_json = serde_json::json!({
        "ok": false,
        "error": {
            "code": code,
            "message": message
        }
    });

    println!("{}", serde_json::to_string_pretty(&error_json).unwrap());
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_error(
            "USAGE_ERROR",
            "Usage: cli.sh <fixture.json> | --block <blk.dat>",
        );
    }

    // ---------------- BLOCK MODE ----------------
if args[1] == "--block" {
    if args.len() != 5 {
        print_error(
            "USAGE_ERROR",
            "Usage: cli.sh --block <blk.dat> <rev.dat> <xor.dat>",
        );
    }

    if let Err(e) = block::run_block_mode(&args[2], &args[3], &args[4]) {
        print_error("BLOCK_ERROR", &e);
    }

    process::exit(0);
}

    // ---------------- TX MODE (DEFAULT) ----------------
    if args.len() != 2 {
        print_error(
            "USAGE_ERROR",
            "Usage: cli.sh <fixture.json> | --block <blk.dat>",
        );
    }

    let file_content = match fs::read_to_string(&args[1]) {
        Ok(content) => content,
        Err(_) => {
            print_error("FILE_READ_ERROR", "Failed to read fixture file");
        }
    };

    let fixture: Fixture = match serde_json::from_str(&file_content) {
        Ok(f) => f,
        Err(_) => {
            print_error("INVALID_JSON", "Invalid fixture JSON");
        }
    };

    let result = match analyze(fixture) {
        Ok(r) => r,
        Err(e) => {
            print_error("ANALYSIS_ERROR", &e);
        }
    };

    let json = serde_json::to_string_pretty(&result).unwrap();

    let out_dir = Path::new("out");
    if !out_dir.exists() {
        if let Err(_) = fs::create_dir(out_dir) {
            print_error("IO_ERROR", "Failed to create out directory");
        }
    }

    let file_path = format!("out/{}.json", result.txid);

    if let Err(_) = fs::write(&file_path, &json) {
        print_error("IO_ERROR", "Failed to write output file");
    }

    println!("{}", json);
}