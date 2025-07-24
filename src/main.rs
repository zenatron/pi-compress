use std::io::{self, Write};
use std::error::Error;

// Compressed segment: either a match in pi or raw bytes.
#[derive(Debug)]
enum Segment {
    Match { pos: usize, len: usize },
    Raw(Vec<u8>),
}

// Embed pi digits at compile time.
fn load_pi() -> &'static str {
    include_str!("pi.txt")
}

// Convert bytes to hex string.
fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// Greedy compress: longest hex‑substring of input found in pi.
fn compress(input: &[u8], pi: &str) -> Vec<Segment> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < input.len() {
        let mut found = false;
        for j in (i+1..=input.len()).rev() {
            let hx = to_hex(&input[i..j]);
            if let Some(p) = pi.find(&hx) {
                out.push(Segment::Match { pos: p, len: j - i });
                i = j;
                found = true;
                break;
            }
        }
        if !found {
            out.push(Segment::Raw(vec![input[i]]));
            i += 1;
        }
    }
    out
}

// Reconstruct original bytes from segments.
fn decompress(segments: &[Segment], pi: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buf = Vec::new();
    for seg in segments {
        match seg {
            Segment::Match { pos, len } => {
                let hex = &pi[*pos..pos + len * 2];
                buf.extend(hex::decode(hex)?);
            }
            Segment::Raw(bytes) => buf.extend(bytes),
        }
    }
    Ok(buf)
}

fn main() -> Result<(), Box<dyn Error>> {
    let pi = load_pi();
    
    loop {
        print!("Enter text to compress (Q to quit): ");
        io::stdout().flush()?;
        
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let input = line.trim();
        
        if input == "Q" {
            break;
        }
        
        let data = input.as_bytes();
        let compressed = compress(data, pi);
        println!("{:?}", compressed);

        let restored = decompress(&compressed, pi)?;
        println!("{}", String::from_utf8(restored)?);
    }
    Ok(())
}
