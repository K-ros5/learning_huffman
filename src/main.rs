use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use clap::{command, Args, Parser};
use learning_huffman::{get_byte_frequencies, CompressFile, DecompressFile};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    compops: CompOps,

    #[command(flatten)]
    decompops: DecompOps,

    ///print frequency of each byte in file
    #[arg(short, long, value_name = "FILE")]
    frequencies: Option<PathBuf>,

    ///output for compress/decompress
    #[arg(value_name = "FILE")]
    output: Option<PathBuf>,
}

#[derive(Args, Debug)]
#[group(requires_all = ["compress", "output"])]
struct CompOps {
    ///compress file
    #[arg(short, long, value_name = "FILE")]
    compress: Option<PathBuf>,
}

#[derive(Args, Debug)]
#[group(requires_all = ["decompress", "output"])]
struct DecompOps {
    ///compress file
    #[arg(short, long, value_name = "FILE")]
    decompress: Option<PathBuf>,
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
    } else if let (Some(input), Some(output)) = (&cli.compops.compress, &cli.output) {
        if let (Some(input), Some(output)) = (input.to_str(), output.to_str()) {
            CompressFile::new().compress(input)?.output_freq(output)?;
        }
    } else if let (Some(input), Some(output)) = (&cli.decompops.decompress, &cli.output) {
        if let (Some(input), Some(output)) = (input.to_str(), output.to_str()) {
            DecompressFile::new()
                .decompress_freq(input)?
                .output(output)?;
        }
    }

    Ok(())
}
