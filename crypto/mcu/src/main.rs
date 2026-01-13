/*
    1. get filename from command line
    2. check if file exist error if not
    3. read buffer
    4. hash buffer
    5. write buffer
    6. repeat step 3 until end of file.
*/

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},  // Add RngCore
    Aes256Gcm, Key, Nonce,
};
use std::{fs};



use blake3::Hasher;
use std::fs::File;
use std::io::{self, BufReader, Read};

fn hash_file(file_path: &str) -> Result<String, io::Error> {
    let file = File::open(file_path)?;

    let mut reader = BufReader::new(file);

    let mut hasher = Hasher::new();

    const BUFFER_SIZE: usize = 4096;
    let mut buffer = vec![0u8; BUFFER_SIZE];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        println!("Read: {} bytes", bytes_read);
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    let hash = hasher.finalize();
    Ok(hash.to_hex().to_string())
}

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mcu", author, version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Encrypt {
        #[arg(short, long)]
        infile: String,
        #[arg(short, long)]
        outfile: String,
    },
    Decrypt {
        #[arg(short, long)]
        infile: String,
        #[arg(short, long)]
        outfile: String,
    },
    Hash {
        #[arg(short, long)]
        infile: String,
    },
}

fn print_hex_dump(label: &str, data: &[u8]) {
    println!("{} ({} bytes):", label, data.len());
    
    for (i, chunk) in data.chunks(16).enumerate() {
        let hex: Vec<_> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
        let hex_str = hex.join(" ");
        
        let ascii: String = chunk.iter()
            .map(|&b| if b.is_ascii_graphic() { b as char } else { '.' })
            .collect();
        
        println!("  {:08X}: {:48} |{:16}|", i * 16, hex_str, ascii);
    }
    
}

fn get_key() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if fs::metadata("key.bin").is_ok() {
        let key = fs::read("key.bin")?;
        if key.len() == 32 { return Ok(key); }
    }
    // Default
    Ok(hex::decode("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")? )
}

fn encrypt_file(infile: &str, outfile: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Encrypting {} → {}", infile, outfile);
    let key_bytes = get_key()?;
    let nonce = Nonce::from_slice(b"mcu_nonce_12");  // Fixed 12 bytes
    
    let plaintext = fs::read(&infile)?;
    print_hex_dump("plaintext", &plaintext);
    
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref()).unwrap();
    
    print_hex_dump("ciphertext", &ciphertext);
    fs::write(&outfile, &ciphertext).unwrap();
    println!("✓ Encrypted: {} bytes", ciphertext.len());
    Ok(())
}

fn decrypt_file(infile: &str, outfile: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Decrypting {} → {}", infile, outfile);
    let key_bytes = get_key()?;
    let nonce = Nonce::from_slice(b"mcu_nonce_12");
    
    let ciphertext = fs::read(&infile)?;
    print_hex_dump("ciphertext", &ciphertext);
    
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref()).unwrap();
    
    print_hex_dump("plaintext", &plaintext);
    fs::write(&outfile, &plaintext).unwrap();
    println!("✓ Decrypted: {} bytes", plaintext.len());
    Ok(())
}
// fn read_buf()

fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let cli = Cli::parse();
    match cli.command {
        Commands::Encrypt { infile, outfile } => {
            encrypt_file(infile.as_str(), outfile.as_str());
        }
        
        Commands::Decrypt { infile, outfile } => {
            decrypt_file(infile.as_str(), outfile.as_str());
        }
        
        Commands::Hash { infile } => {
            println!("Hashing {}", infile);
            let hash = hash_file(&infile)?;
            println!("Blake3: {}", hash);
        }
    }

    Ok(())
}
