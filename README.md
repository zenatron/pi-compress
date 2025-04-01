# PiCompress: The Future of Data Compression and Encryption?

## Overview
Welcome to $\pi$Compress, a revolutionary data "compression" tool developed as a fun thought experiment. Leveraging Rust's speed and safety, $\pi$Compress uses the first million digits of $\pi$ to "compress" data. Like other compression algorithms, it works super fast and is super secure (unless you need to compress more than a few bytes). On top of that, the output is even larger than the input! Checkmate, [LZMA](https://en.wikipedia.org/wiki/LZMA)!

## The "Algorithm"
The core idea is based on the fascinating (and mathematically unproven for *all* sequences) concept that the digits of $\pi$ contain every possible finite sequence of numbers.

$\pi$Compress works like this:
1.  **Hexadecimal Conversion:** Taking your input text.
2.  **Byte-to-Hex Encoding:** Converting each byte of the text into its two-digit hexadecimal representation (e.g., 'H' -> 0x48 becomes "48").
3.  **The Pi Search:** Searching for this resulting hexadecimal string within a stored sequence of the **first million digits of Pi** (from `pi.txt`). Note that since Pi only contains digits 0-9, this search will only ever find hex strings that happen to *only* contain those digits (e.g., "1234" might be found, but "4a8b" never will).
4.  **Greedy Longest Match:** It tries to find the longest possible chunk of your input text (starting from the beginning) whose hex representation exists in Pi.
5.  **Verification:** Crucially, it verifies that the specific sequence found in Pi can be reliably converted *back* into the original bytes (this step prevents some hilarious but incorrect results).
6.  **"Compressed" Output:** The output isn't actually smaller! It's a list of instructions:
    *   `Pi[index] (N bytes)`: Meaning "fetch the data representing N original bytes starting at this index in Pi".
    *   `Raw[0xHH...]`: Meaning "this byte couldn't be reliably found in Pi, so here's its original hex value".
7.  **Decompression:** Reverses the process by fetching the digits from the stored Pi sequence based on the indices and converting them back to text (via hex), or using the raw bytes directly.

## Why?
Mostly for fun and to explore a quirky idea. This is **not** a practical compression algorithm. In fact, the "compressed" representation is often significantly larger than the original input due to storing indices and raw data. Clone the repo and run it to see for yourself!

## Disclaimer
This project is intended for educational and entertainment purposes only. Do not use it for any serious data compression needs (or do, I guess). Happy April Fools'!