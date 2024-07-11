use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read, Write},
    marker::PhantomData,
};

use crate::{huff::get_byte_frequencies, HuffCode, HuffNode};

///Compress a file using simple Hoffman Code
#[derive(Debug)]
pub struct CompressFile<S: CompState> {
    state: Box<ActualCompState>,

    _marker: std::marker::PhantomData<S>,
}

pub enum StartComp {}
pub enum Compress {}
pub enum OutputComp {}

pub trait CompState {}
impl CompState for StartComp {}
impl CompState for Compress {}
impl CompState for OutputComp {}

#[derive(Debug)]
struct ActualCompState {
    compressed_bytes: Vec<u8>,
    compressed_last_byte_length: u8,
    frequencies: [usize; 256],
}

///initialize compression
impl CompressFile<StartComp> {
    pub fn new() -> CompressFile<Compress> {
        CompressFile {
            state: Box::new(ActualCompState {
                compressed_bytes: vec![],
                compressed_last_byte_length: 0,
                frequencies: [0; 256],
            }),
            _marker: PhantomData,
        }
    }
}

///performs the actual compress
impl CompressFile<Compress> {
    ///returns a tuple of (compressed_bytes, size of the last byte)
    fn compress_bytes(
        &mut self,
        uncompressed_bytes: &[u8],
        frequencies: &[usize; 256],
    ) -> (Vec<u8>, u8) {
        let mut compressed_bytes = Vec::new();
        let huff = HuffNode::from_frequencies(frequencies);

        let table = HuffCode::from_tree(&huff);

        let mut compressed_byte = 0;
        let mut b_index = 0;

        let mut has_remaining = false;

        for byte in uncompressed_bytes {
            let (code, length) = match table.get(byte) {
                Some(item) => (item.get_code(), item.get_length()),
                None => panic!("HashMap table key doesn't exist!"),
            };

            for i in 0..length {
                let code_bit: u8 = ((code >> i) & 1).try_into().unwrap();

                //There's a possibility bits remain in `compressed_byte`` when
                //processing final `byte`
                has_remaining = true;

                compressed_byte |= code_bit << b_index;

                if b_index == 7 {
                    compressed_bytes.push(compressed_byte);
                    b_index = 0;
                    compressed_byte = 0;
                    has_remaining = false;
                } else {
                    b_index += 1;
                }
            }
        }

        if has_remaining {
            compressed_bytes.push(compressed_byte);
        }

        (compressed_bytes, b_index)
    }

    pub fn compress(mut self, file: &str) -> io::Result<CompressFile<OutputComp>> {
        let mut file = File::open(file)?;
        let mut uncompressed_bytes = Vec::new();
        file.read_to_end(&mut uncompressed_bytes)?;

        let frequencies = get_byte_frequencies(&uncompressed_bytes);

        let (compressed_bytes, last_byte_len) =
            self.compress_bytes(&uncompressed_bytes, &frequencies);
        self.state.frequencies = frequencies;
        self.state.compressed_bytes = compressed_bytes;
        self.state.compressed_last_byte_length = last_byte_len;

        Ok(CompressFile {
            state: self.state,
            _marker: PhantomData,
        })
    }
}

///compression output implementations
impl CompressFile<OutputComp> {
    ///create frequency header using state data
    fn create_freq_header(&self) -> Vec<u8> {
        let mut compressed_bytes: Vec<u8> = Vec::new();
        let mut freq_list = Vec::new();

        for (byte, freq) in self.state.frequencies.iter().enumerate() {
            if *freq != 0 {
                freq_list.push((byte as u8, *freq));
            }
        }

        //table_size = remaining bit size + (valid frequency * (byte size + frequency weight size))
        let table_size = 1 + (freq_list.len() * (1 + 8));

        compressed_bytes.extend_from_slice(&table_size.to_be_bytes());

        compressed_bytes.extend_from_slice(&self.state.compressed_last_byte_length.to_be_bytes());

        for (byte, freq) in freq_list {
            compressed_bytes.extend_from_slice(&byte.to_be_bytes());
            compressed_bytes.extend_from_slice(&freq.to_be_bytes());
        }

        compressed_bytes
    }

    ///output with frequency header table to `file`
    /// this is a ~very~ stupid implementation as the header will be LARGE
    pub fn output_freq(&mut self, file: &str) -> io::Result<()> {
        let mut file = File::create_new(file)?;
        let header = self.create_freq_header();

        file.write_all(&header)?;
        file.write_all(&self.state.compressed_bytes)?;

        Ok(())
    }
}

//...A lot of copied behavior

///Compress a file using simple Hoffman Code
#[derive(Debug)]
pub struct DecompressFile<S: DecompState> {
    state: Box<ActualDecompState>,

    _marker: std::marker::PhantomData<S>,
}

pub enum StartDecomp {}
pub enum Decompress {}
pub enum OutputDecomp {}

pub trait DecompState {}
impl DecompState for StartDecomp {}
impl DecompState for Decompress {}
impl DecompState for OutputDecomp {}

#[derive(Debug)]
struct ActualDecompState {
    uncompressed_bytes: Vec<u8>,
    //frequencies: [usize; 256],
}

///initialize compression
impl DecompressFile<StartDecomp> {
    pub fn new() -> DecompressFile<Decompress> {
        DecompressFile {
            state: Box::new(ActualDecompState {
                uncompressed_bytes: vec![],
                //frequencies: [0; 256],
            }),
            _marker: PhantomData,
        }
    }
}

///perform actual decompression
/// unlike compressing, decompressing has to use the file headers to achieve anything
impl DecompressFile<Decompress> {
    fn decompress_bytes(
        &self,
        bytes: &[u8],
        frequencies: &[usize; 256],
        last_byte_size: u8,
    ) -> Vec<u8> {
        let mut decompressed_bytes = Vec::new();
        let huff = HuffNode::from_frequencies(frequencies);

        let table = HuffCode::from_tree(&huff);

        let mut new_table: HashMap<(u16, u128), u8> = HashMap::new();

        for (key, value) in table.iter() {
            new_table.insert((value.get_length() - 1, value.get_code()), *key);
        }

        let bytes_length = bytes.len();

        let mut remaining_bit_size = 8;

        let mut current_code: u128 = 0;
        let mut current_length: u16 = 0;

        for (i, byte) in bytes.iter().enumerate() {
            if i == bytes_length - 1 {
                //current byte is the last byte

                remaining_bit_size = last_byte_size;
            }
            for i in 0..remaining_bit_size {
                let code_bit: u128 = ((byte >> i) & 1).into();
                current_code |= code_bit << current_length;

                if let Some(b) = new_table.get(&(current_length, current_code)) {
                    decompressed_bytes.push(*b);
                    current_length = 0;
                    current_code = 0;
                } else {
                    current_length += 1;
                }
            }
        }

        decompressed_bytes
    }

    pub fn decompress_freq(mut self, file: &str) -> io::Result<DecompressFile<OutputDecomp>> {
        let mut file = File::open(file)?;
        let mut frequencies = [0; 256];

        let mut table_size = [0; 8];

        file.read_exact(&mut table_size)?;

        let mut table_size = usize::from_be_bytes(table_size);

        let mut last_byte_size = [0];

        file.read_exact(&mut last_byte_size)?;

        table_size -= 1;

        while table_size > 0 {
            let mut byte = [0];
            file.read_exact(&mut byte)?;

            let mut frequency = [0; 8];
            file.read_exact(&mut frequency)?;

            frequencies[byte[0] as usize] = usize::from_be_bytes(frequency);

            table_size -= 9;
        }

        let mut compressed_bytes: Vec<u8> = Vec::new();
        file.read_to_end(&mut compressed_bytes)?;

        self.state.uncompressed_bytes =
            self.decompress_bytes(&compressed_bytes, &frequencies, last_byte_size[0]);

        Ok(DecompressFile {
            state: self.state,
            _marker: PhantomData,
        })
    }
}

impl DecompressFile<OutputDecomp> {
    ///output decompressed file
    pub fn output(&mut self, file: &str) -> io::Result<()> {
        let mut file = File::create_new(file)?;
        file.write_all(&self.state.uncompressed_bytes)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_bytes_test() {
        let bytes = vec![b'A', b'A', b'C', b'D'];
        let frequencies = get_byte_frequencies(&bytes);
        let (mut comp, size) = CompressFile::new().compress_bytes(&bytes, &frequencies);
        assert_eq!(comp.pop().unwrap(), 0b00011100);
        assert_eq!(size, 6);
    }

    #[test]
    fn create_freq_header_test() {
        let bytes = vec![b'A', b'A', b'C', b'D'];
        let frequencies = get_byte_frequencies(&bytes);
        let (comp, size) = CompressFile::new().compress_bytes(&bytes, &frequencies);
        let c: CompressFile<OutputComp> = CompressFile {
            state: Box::new(ActualCompState {
                compressed_bytes: comp,
                compressed_last_byte_length: size,
                frequencies,
            }),
            _marker: PhantomData,
        };

        let output = [
            0, 0, 0, 0, 0, 0, 0, 28, 6, 65, 0, 0, 0, 0, 0, 0, 0, 2, 67, 0, 0, 0, 0, 0, 0, 0, 1, 68,
            0, 0, 0, 0, 0, 0, 0, 1,
        ];

        assert!(c.create_freq_header().eq(&output));
    }

    #[test]
    fn decompress_bytes_test() {
        let bytes = vec![b'A', b'A', b'C', b'D'];
        let frequencies = get_byte_frequencies(&bytes);
        let (comp, size) = CompressFile::new().compress_bytes(&bytes, &frequencies);
        let decompressed = DecompressFile::new().decompress_bytes(&comp, &frequencies, size);

        assert_eq!(bytes, decompressed);
    }
}
