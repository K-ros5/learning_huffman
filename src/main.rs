use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use clap::{command, Args, Parser};
use learning_huffman::{get_byte_frequencies, CompressFile};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    ops: Ops,

    ///print frequency of each byte in file
    #[arg(short, long, value_name = "FILE")]
    frequencies: Option<PathBuf>,
}

#[derive(Args, Debug)]
#[group(requires_all = ["compress", "output"])]
struct Ops {
    ///compress file
    #[arg(short, long, value_name = "FILE")]
    compress: Option<PathBuf>,

    ///output file
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

fn get_file_bytes(path: &PathBuf) -> Vec<u8> {
    let mut file = File::open(path).expect("Couldn't open file.");
    let mut bytes: Vec<u8> = Vec::new();
    file.read_to_end(&mut bytes).expect("Couldn't read file.");

    bytes
}

fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();

    if let Some(path) = cli.frequencies {
        let bytes = get_file_bytes(&path);
        println!("{:?}", get_byte_frequencies(&bytes));
    }

    if let (Some(input), Some(output)) = (cli.ops.compress, cli.ops.output) {
        if let (Some(input), Some(output)) = (input.to_str(), output.to_str()) {
            CompressFile::new().compress(input)?.output_freq(output)?;
        }
    }

    Ok(())
}
