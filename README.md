# learning_huffman

Further learning by writing a huffman encoder/decoder

## Challenge
The idea for this "challenge" was taken from [codingchallengs.fyi](https://codingchallenges.fyi/challenges/challenge-huffman/).
As suggested by codingchallenges, the test file was taken from [here](https://www.gutenberg.org/files/135/135-0.txt).

## Info
Currently uses a VERY dumb implementation for the file header output, using byte 

## Usage
```
Usage: learning_huffman [OPTIONS] [FILE]

Arguments:
  [FILE]  output file

Options:
  -c, --compress <FILE>     compress file
  -d, --decompress <FILE>   compress file
  -f, --frequencies <FILE>  print frequency of each byte in file
  -h, --help                Print help
  -V, --version             Print version
```

## Todo
* Implement better header output type, not using frequencies