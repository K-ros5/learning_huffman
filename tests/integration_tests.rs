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
        let response = Command::new("./target/debug/learning_huffman")
            .arg("-c")
            .arg("./test_files/135-0.txt")
            .arg("-o")
            .arg("./test_output/compressed.txt")
            .output()
            .expect("Binary not found?")
            .stdout;

        let mut file =
            File::open("./test_output/compressed.txt").expect("Couldn't open compressed txt");

        file.seek(SeekFrom::Start(9 + 1)).expect("couldn't seek");

        let mut freq = [0; 8];

        file.read_exact(&mut freq).expect("coudn't read_exact");

        freq.reverse();

        assert_eq!(73589, usize::from_le_bytes(freq));
    }
}
