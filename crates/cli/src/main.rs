use std::{fs, path::PathBuf};

use log::{debug, error, info};
use my_lib::Module;
use structopt::{clap::AppSettings, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(
    name = "wasm-runner",
    about = "Sample project from https://vino.dev/blog/node-to-rust-day-1-rustup/",
    global_settings(&[
      AppSettings::ColoredHelp
    ]),
)]
struct CliOptions {
    /// The WebAssembly file to load, try "crates/my-lib/tests/wapc_guest.wasm" or "./blog.wasm"
    #[structopt(parse(from_os_str))]
    pub(crate) file_path: PathBuf,

    /// The operation to invoke in the WASM file, try "hello" or "render"
    #[structopt()]
    pub(crate) operation: String,

    /// The path to the JSON data to use as input. Try "./hello.json" or "./blog.json"
    #[structopt(parse(from_os_str))]
    pub(crate) json_path: PathBuf,
}

/// Usage:
/// ```bash
/// $ cargo run crates/my-lib/tests/wapc_guest.wasm hello hello.json
/// $ cargo run -r ./blog.wasm render ./blog.json # Release build
/// ```
fn main() {
    env_logger::init();
    debug!("Initialized logger");
    let options = CliOptions::from_args();

    match run(options) {
        Ok(output) => {
            println!("{}", output);
            info!("Done");
        }
        Err(e) => {
            error!("Module failed to load: {}", e);
            std::process::exit(1);
        }
    };
}

fn run(options: CliOptions) -> anyhow::Result<serde_json::Value> {
    let module = Module::from_file(&options.file_path)?;
    info!("Module loaded");

    let json = fs::read_to_string(options.json_path)?;
    let data: serde_json::Value = serde_json::from_str(&json)?;
    debug!("Data: {:?}", data);
    let bytes = rmp_serde::to_vec(&data)?;

    debug!("Running  {} with payload: {:?}", options.operation, bytes);
    let result = module.run(&options.operation, &bytes)?;
    let unpacked = rmp_serde::from_slice(&result)?;

    Ok(unpacked)
}
