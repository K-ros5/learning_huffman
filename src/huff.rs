use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HuffNode {
    weight: usize,
    byte: Option<u8>,
    left: Option<Box<HuffNode>>,
    right: Option<Box<HuffNode>>,
}

impl PartialOrd for HuffNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HuffNode {
    fn cmp(&self, other: &Self) -> Ordering {
        //You need to compare the byte as well or ordering can become equal and
        //BinarHeap will give back equal items in random order
        other
            .weight
            .cmp(&self.weight)
            .then(self.byte.cmp(&other.byte))
    }
}

impl HuffNode {
    pub fn from_frequencies(frequencies: &[usize; 256]) -> Option<Box<Self>> {
        let mut heap = BinaryHeap::new();

        for (i, frequency) in frequencies.iter().enumerate() {
            if *frequency != 0 {
                heap.push(HuffNode {
                    weight: *frequency,
                    byte: Some(i as u8),
                    left: None,
                    right: None,
                });
            }
        }

        while heap.len() > 1 {
            let mut new_node = HuffNode {
                weight: 0,
                byte: None,
                left: None,
                right: None,
            };

            if let (Some(left), Some(right)) = (heap.pop(), heap.pop()) {
                new_node.weight = left.weight + right.weight;
                new_node.left = Some(Box::new(left));
                new_node.right = Some(Box::new(right));
                heap.push(new_node);
            }
        }

        heap.pop().map(Box::new)
    }
}

pub struct HuffCode {
    weight: usize,
    length: u16,
    code: u128,
}

impl std::fmt::Debug for HuffCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("HuffCode")
            .field("weight", &format_args!("{}", &self.weight))
            .field("length", &format_args!("{}", &self.length))
            .field("code", &format_args!("{:08b}", &self.code))
            .finish()
    }
}

impl HuffCode {
    fn generate_codes(
        node: &Option<Box<HuffNode>>,
        table: &mut HashMap<u8, HuffCode>,
        code: u128,
        shift: u16,
    ) {
        if let Some(node) = node {
            if let Some(byte) = node.byte {
                table.insert(
                    byte,
                    HuffCode {
                        weight: node.weight,
                        length: shift,
                        code,
                    },
                );
                return;
            }

            //left
            Self::generate_codes(&node.left, table, code | (0 << shift), shift + 1);

            //right
            Self::generate_codes(&node.right, table, code | (1 << shift), shift + 1);
        }
    }

    pub fn from_tree(node: &Option<Box<HuffNode>>) -> HashMap<u8, Self> {
        let mut lookup_table = HashMap::new();
        Self::generate_codes(node, &mut lookup_table, 0, 0);
        lookup_table
    }

    pub fn get_code(&self) -> u128 {
        self.code
    }

    pub fn get_length(&self) -> u16 {
        self.length
    }
}

pub fn get_byte_frequencies(bytes: &[u8]) -> [usize; 256] {
    let mut frequencies = [0; 256];
    for byte in bytes {
        frequencies[*byte as usize] += 1;
    }

    frequencies
}

#[allow(non_snake_case)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_byte_frequencies_test() {
        let bytes = vec![b'A', b'A', b'C', b'D'];
        assert!(get_byte_frequencies(&bytes)[b'A' as usize] == 2);
    }

    #[test]
    fn HuffNode_from_frequencies_test() {
        let bytes = vec![b'A', b'A', b'C', b'D'];
        let frequencies = get_byte_frequencies(&bytes);
        let node = HuffNode::from_frequencies(&frequencies);

        if let Some(node) = node {
            if let Some(left) = node.left {
                let isA = match left.byte {
                    None => false,
                    Some(byte) => byte == b'A',
                };

                if !isA {
                    panic!("First left node should be A!");
                }
            }
        } else {
            panic!("Node not initialized");
        }
    }

    #[test]
    fn HuffCode_from_tree() {
        let bytes = vec![b'A', b'A', b'C', b'D'];
        let frequencies = get_byte_frequencies(&bytes);
        let node = HuffNode::from_frequencies(&frequencies);

        let table = HuffCode::from_tree(&node);

        assert_eq!(table.get(&b'A').unwrap().length, 1);
        assert_eq!(table.get(&b'C').unwrap().length, 2);
        assert_eq!(table.get(&b'D').unwrap().length, 2);

        assert_eq!(table.get(&b'A').unwrap().code, 0);
        assert_eq!(table.get(&b'C').unwrap().code, 0b00000011);
        assert_eq!(table.get(&b'D').unwrap().code, 0b00000001);
    }

    #[test]
    fn test_BinaryHeap_ord() {
        let mut heap = BinaryHeap::new();

        heap.push(HuffNode {
            weight: 1,
            byte: Some(b'C'),
            left: None,
            right: None,
        });

        heap.push(HuffNode {
            weight: 2,
            byte: Some(b'A'),
            left: None,
            right: None,
        });

        heap.push(HuffNode {
            weight: 1,
            byte: Some(b'D'),
            left: None,
            right: None,
        });

        //First off should have the lowest weights
        if let Some(node) = heap.pop() {
            if node.byte.unwrap() != b'D' {
                panic!("Not correct pop order C - was {}", node.byte.unwrap());
            }
        }

        if let Some(node) = heap.pop() {
            if node.byte.unwrap() != b'C' {
                panic!("Not correct pop order C - was {}", node.byte.unwrap());
            }
        }

        if let Some(node) = heap.pop() {
            if node.byte.unwrap() != b'A' {
                panic!("Not correct pop order C - was {}", node.byte.unwrap());
            }
        }
    }
}
