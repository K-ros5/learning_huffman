#[cfg(test)]
mod test {
    use std::{
        fs::File,
        io::{Read, Seek, SeekFrom},
        process::Command,
    };

    #[test]
    fn get_byte_frequencies_file() {
        let response = Command::new("./target/debug/learning_huffman")
            .arg("-f")
            .arg("./test_files/135-0.txt")
            .output()
            .expect("Binary not found?")
            .stdout;

        let command_string = String::from_utf8(response).unwrap();

        assert!(command_string.contains("157632, 223000, 67391"));
    }

    #[test]
    fn compress_file_freq_header() {
        Command::new("./target/debug/learning_huffman")
            .arg("-c")
            .arg("./test_files/135-0.txt")
            .arg("-o")
            .arg("./test_output/compressed.txt")
            .output()
            .expect("Binary not found?");

        let mut file =
            File::open("./test_output/compressed.txt").expect("Couldn't open compressed txt");

        file.seek(SeekFrom::Start(9 + 1)).expect("couldn't seek");

        let mut freq = [0; 8];

        file.read_exact(&mut freq).expect("coudn't read_exact");

        freq.reverse();

        assert_eq!(73589, usize::from_le_bytes(freq));
    }

    #[test]
    fn test_compression_decompression() {
        Command::new("./target/debug/learning_huffman")
            .arg("-c")
            .arg("./test_files/135-0.txt")
            .arg("./test_output/compressed.txt")
            .output()
            .expect("Binary not found?");

        Command::new("./target/debug/learning_huffman")
            .arg("-d")
            .arg("./test_output/compressed.txt")
            .arg("./test_output/decompressed.txt")
            .output()
            .expect("Binary not found?");

        let mut file1 = File::open("./test_files/135-0.txt").expect("Couldn't open compressed txt");

        let mut file2 =
            File::open("./test_output/decompressed.txt").expect("Couldn't open decompressed txt");

        let mut comp = Vec::new();
        let mut decomp = Vec::new();
        file1.read_to_end(&mut comp).expect("couldnt read to end");
        file2.read_to_end(&mut decomp).expect("couldnt read to end");

        assert!(comp.eq(&decomp));
    }
}
