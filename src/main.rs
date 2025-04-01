use std::io;

// --- Data Structures ---

/// Represents the result of trying to "compress" a chunk of the input.
#[derive(Debug)] // Allow printing the enum for debugging.
enum MatchResult {
    /// Indicates a chunk of the original input was successfully found in Pi.
    Found { 
        index: usize,             // The starting position (0-based index) in the Pi string.
        hex_len: usize,           // The length of the HEXADECIMAL digit sequence found in Pi.
        original_byte_len: usize  // The number of original bytes this sequence represents.
    },
    /// Indicates a byte from the original input could not be reliably found in Pi.
    NotFound { 
        data: Vec<u8>             // Contains the original byte(s) that were not found.
    },
}

// --- Core Logic Functions ---

/// Loads the Pi digits from the embedded file `pi.txt` at compile time.
/// Returns a static string slice (`&'static str`) containing the digits.
/// `include_str!` embeds the file content directly into the binary.
fn load_pi_digits() -> &'static str {
    // Assumes pi.txt is in the same directory as this source file (src/).
    include_str!("pi.txt") 
}

/// Converts a slice of bytes into its hexadecimal string representation.
/// Example: &[0x48, 0x65] ('H', 'e') -> "4865"
fn bytes_to_hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b)) // Convert each byte to its 2-digit lowercase hex string.
        .collect() // Collect into the final hex string.
}

/// Converts a hexadecimal string back into its original byte sequence.
/// Reverses `bytes_to_hex_string`.
/// Handles errors like invalid hex characters or an odd number of hex digits.
fn hex_string_to_bytes(hex_str: &str) -> Result<Vec<u8>, String> {
    // Ensure the hex string has an even number of characters for valid byte pairing.
    if hex_str.len() % 2 != 0 {
        return Err(format!("Hex string has odd length: '{}'", hex_str));
    }

    let mut bytes = Vec::with_capacity(hex_str.len() / 2);
    // Iterate over the hex string in chunks of 2 characters.
    for i in (0..hex_str.len()).step_by(2) {
        // Extract the 2-character hex representation of a byte.
        let hex_pair = &hex_str[i..i + 2];
        // Parse the hex pair into a u8 byte.
        match u8::from_str_radix(hex_pair, 16) {
            Ok(byte) => bytes.push(byte), // Add the byte to the result vector.
            Err(_) => return Err(format!("Invalid hex sequence '{}'", hex_pair)), // Error on invalid hex pair.
        }
    }

    Ok(bytes) // Return the successfully decoded bytes.
}

// Helper function for `compress`: Tries to find the hex representation of a byte slice in Pi
// and verifies if that *specific* found Pi slice converts back to the *original* bytes.
fn find_and_verify_match(
    slice_to_try: &[u8], 
    pi: &str             
) -> Option<(usize, usize)> { // Returns Some(index_in_pi, length_of_hex_string) if verified.
    
    // 1. Convert the original bytes to the target hex string format.
    let hex_str_to_find = bytes_to_hex_string(slice_to_try);

    // 2. Search for the *first* occurrence of this hex string within Pi.
    //    NOTE: Pi itself only contains decimal digits '0'-'9'. Searching for hex 'a'-'f' will always fail.
    //    This whole approach relies on finding sequences in Pi that *happen* to look like hex strings.
    if let Some(index) = pi.find(&hex_str_to_find) {
        let hex_len = hex_str_to_find.len(); // Length of the hex string found in Pi.

        // 3. Basic sanity check: Ensure the found match doesn't extend beyond Pi's bounds.
        if index + hex_len <= pi.len() {
            
            // 4. Extract the exact slice from Pi that matched.
            let pi_slice = &pi[index..index + hex_len];
            
            // 5. Attempt to convert this specific Pi slice (as a hex string) back to bytes.
            if let Ok(decoded_bytes) = hex_string_to_bytes(pi_slice) {
                
                // 6. THE CRITICAL VERIFICATION: Does the result of decoding the Pi slice 
                //    exactly match the original bytes we started with?
                if decoded_bytes == slice_to_try {
                    // Yes! This is a valid, reversible match.
                    return Some((index, hex_len));
                }
                // Else: Decoded bytes didn't match original `slice_to_try`.
            }
            // Else: `hex_string_to_bytes` failed for this `pi_slice` (invalid hex chars).
        }
        // Else: The found match `index + hex_len` would be out of Pi bounds.
    }
    
    // No verified match found.
    None
}

/// Compresses the input text using a greedy longest-match strategy against Pi digits (using hex representation).
/// It iterates through the input, trying to find the longest possible chunk from the 
/// current position whose hex representation can be found *and verified* in Pi.
fn compress(text: &str, pi: &str) -> Vec<MatchResult> {
    let mut results = Vec::new(); // Stores the sequence of Found/NotFound results.
    let input_bytes = text.as_bytes(); // Work with the input as bytes.
    let mut current_pos = 0; // Tracks our position in the input_bytes.

    // Process the input bytes chunk by chunk.
    while current_pos < input_bytes.len() {
        let mut longest_verified_match_found = false;

        // Inner loop: Try matching substrings starting at `current_pos`.
        // We iterate `len` from the longest possible remaining substring down to 1 byte.
        for len in (1..=input_bytes.len() - current_pos).rev() {
            // Get the current byte slice to attempt matching.
            let slice_to_try = &input_bytes[current_pos..current_pos + len];

            // Use the helper function to find AND verify the match in Pi.
            if let Some((index, hex_len)) = find_and_verify_match(slice_to_try, pi) {
                // Success! A verified match was found.
                results.push(MatchResult::Found { 
                    index,              // Index in Pi.
                    hex_len,        // Length of the hex string found.
                    original_byte_len: len // Length of the original bytes matched.
                });
                current_pos += len; // Advance our position in the input by the matched length.
                longest_verified_match_found = true;
                // Since we iterate `len` from longest to shortest, the first verified match 
                // we find is guaranteed to be the longest possible for this `current_pos`.
                // So, we can break the inner loop and move to the next position.
                break; 
            }
            // Else: This `len` didn't yield a verified match. The loop continues to try the next shorter length.
        }

        // If the inner loop completed without finding any verified match (even for len=1),
        // it means the single byte at `current_pos` could not be found and verified in Pi.
        if !longest_verified_match_found {
            // Record this byte as NotFound.
            let byte = input_bytes[current_pos];
            results.push(MatchResult::NotFound { data: vec![byte] });
            current_pos += 1; // Advance the position by just one byte.
        }
    }
    results // Return the collected sequence of match results.
}

/// Decompresses a sequence of `MatchResult` items back into the original string.
fn decompress(matches: &[MatchResult], pi: &str) -> Result<String, String> {
    let mut result_bytes = Vec::new(); // Accumulates the reconstructed bytes.

    // Iterate through the compression results.
    for result in matches {
        match result {
            // If the chunk was found in Pi:
            MatchResult::Found { index, hex_len, original_byte_len } => {
                // Sanity check: Ensure the stored index/length are within Pi's bounds.
                if *index + *hex_len > pi.len() {
                     return Err(format!("Invalid index/hex_len leads out of Pi bounds: index={}, len={}, pi_len={}", index, hex_len, pi.len()));
                }
                // Extract the required digit sequence from Pi.
                let pi_slice = &pi[*index..*index + *hex_len];
                // Convert the Pi slice (interpreted as a hex string) back to bytes.
                match hex_string_to_bytes(pi_slice) {
                    Ok(bytes) => {
                        // **Verification during Decompression (Safety Check)**
                        // Although `compress` already verified, we double-check here 
                        // to ensure the decoded byte count matches expectations.
                        // This guards against potential logic errors or corrupted `MatchResult` data.
                        if bytes.len() == *original_byte_len {
                            result_bytes.extend(bytes); // Append the decoded bytes.
                        } else {
                            // This indicates an internal inconsistency - the Pi slice didn't decode
                            // to the expected number of bytes, despite compress verifying it.
                            return Err(format!(
                                "Decompression mismatch: Pi slice (hex) '{}' ({}) converted to {} bytes, expected {}",
                                pi_slice,
                                hex_len,
                                bytes.len(),
                                original_byte_len
                            ));
                        }
                    },
                    // Handle errors during the Pi slice -> bytes conversion.
                    Err(e) => return Err(format!("Error converting Pi slice (hex) back to bytes: {}", e)),
                }
            }
            // If the chunk was not found, use the stored raw byte(s).
            MatchResult::NotFound { data } => {
                result_bytes.extend(data); // Append the original byte(s) directly.
            }
        }
    }
    // Convert the final accumulated byte vector back to a UTF-8 string.
    // This can fail if the original input wasn't valid UTF-8 or if reconstruction somehow failed.
    String::from_utf8(result_bytes).map_err(|e| format!("Decompressed bytes are not valid UTF-8: {}", e))
}

// --- Main Program Execution ---

fn main() {
    // Load the Pi digits once at the start.
    let pi = load_pi_digits();

    // --- Get User Input ---
    println!("Enter the text to compress:");
    let mut input_str = String::new(); // Buffer for stdin.
    io::stdin()
        .read_line(&mut input_str) // Read a line from the console.
        .expect("Failed to read line"); // Basic error handling for read failure.
    let input = input_str.trim(); // Remove leading/trailing whitespace (like the newline).

    println!("Original: {}\n", input);

    // --- Compression ---
    // Run the main compression logic.
    let compressed_data: Vec<MatchResult> = compress(input, pi);
    
    // --- Print Compressed Output ---
    // Display the results in a user-friendly format.
    println!("Compressed representation:");
    print!("  "); // Indent for readability.
    for (i, result) in compressed_data.iter().enumerate() {
        if i > 0 { print!(", "); } // Add separators between results.
        match result {
            // Format for found chunks: Pi[index] (N bytes)
            MatchResult::Found { index, original_byte_len, .. } => {
                print!("Pi[{}] ({} bytes)", index, original_byte_len);
            }
            // Format for not found bytes: Raw[0xHH...]
            MatchResult::NotFound { data } => {
                // Convert the raw byte(s) to a concise hex string.
                let hex_data = data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join("");
                print!("Raw[0x{}]", hex_data);
            }
        }
    }
    println!(); // Add a newline after the compressed output.

    // --- Decompression ---
    // Attempt to decompress the data back.
    match decompress(&compressed_data, pi) {
        Ok(decompressed) => {
            println!("\nDecompressed: {}\n", decompressed);
            // --- Verification ---
            // Check if the decompressed result matches the original input.
            if input == decompressed {
                println!("Verification successful!");
            } else {
                // This should ideally not happen if the logic is correct.
                println!("Verification FAILED!");
                println!("Original len: {}, Decompressed len: {}", input.len(), decompressed.len());
            }
        }
        // Handle potential errors during decompression.
        Err(e) => {
            println!("Decompression failed: {}", e);
        }
    }
}