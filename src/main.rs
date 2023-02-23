#![allow(unused_imports)]
use std::{io::Read, path::PathBuf};

use binary_rw::{BinaryError, BinaryReader, FileStream};
use log::{debug, error, info, trace, warn};
use thiserror::Error;

#[cfg(target_os = "linux")]
use progress_bar::*;

#[derive(Debug, Error)]
enum TModError {
    #[error("No input file given")]
    NoInputFile,
    #[error("No output directory given")]
    NoOutputDirectory,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Read error: {0}")]
    ReadError(#[from] binary_rw::BinaryError),
    #[error("UTF8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

struct ModFile {
    pub name: String,
    pub uncompressed_len: i32,
    pub compressed_len: i32,
}

fn main() {
    env_logger::init();

    if let Err(e) = unpack() {
        eprintln!("Error: {}", e);
        if matches!(e, TModError::NoInputFile | TModError::NoOutputDirectory) {
            println!();
            show_usage();
        }
    }
}

fn show_usage() {
    println!("Usage: tmod-extract <input file> <output directory>");
    println!("Extracts the contents of a tModLoader mod file.\n");
    println!("Set the RUST_LOG environment variable to set the log level.");
}

fn unpack() -> Result<(), TModError> {
    let mut args = std::env::args();
    let _ = args.next();

    let path = args.next().ok_or(TModError::NoInputFile)?;
    if path == "-h" || path == "--help" {
        show_usage();
        return Ok(());
    }

    let out_dir = args.next().ok_or(TModError::NoOutputDirectory)?;
    let out_dir = PathBuf::from(out_dir);

    trace!("checking if output directory exists: {:?}", out_dir);
    if !out_dir.try_exists()? {
        trace!("output directory does not exist, creating it");
        std::fs::create_dir_all(&out_dir)?;
    }

    trace!("opening file: {}", path);
    let file = std::fs::File::open(path)?;
    let mut stream = FileStream::new(file);
    let mut reader = BinaryReader::new(&mut stream, binary_rw::Endian::Little);

    trace!("reading header");
    let header = reader.read_bytes(4)?;
    assert_eq!(b"TMOD", &header[..]);
    debug!("TMOD header found");

    trace!("reading tmodloader version");
    let tmodloader_version = read_csharp_string(&mut reader)?;
    info!("For tModLoader version: {}", tmodloader_version);

    trace!("reading mod hash");
    let hash = reader.read_bytes(20)?;
    let hash_str = hex::encode(hash);
    debug!("Hash: {}", hash_str);

    trace!("reading signature");
    let signature = reader.read_bytes(256)?;
    let signature_str = hex::encode(signature);
    debug!("Signature: {}", signature_str);

    trace!("reading file data length");
    let file_data_len = reader.read_u32()?;
    debug!("File data length: {}", file_data_len);

    trace!("reading mod name");
    let mod_name = read_csharp_string(&mut reader)?;
    info!("Mod name: {}", mod_name);

    trace!("reading mod version");
    let mod_version = read_csharp_string(&mut reader)?;
    info!("Mod version: {}", mod_version);

    trace!("reading file count");
    let file_count = reader.read_i32()?;
    debug!("File count: {}", file_count);

    let mut file_entries = Vec::with_capacity(file_count as usize);

    #[cfg(target_os = "linux")]
    {
        init_progress_bar(file_count as usize * 2);
        set_progress_bar_action("Reading", Color::Blue, Style::Bold);
    }

    info!("Reading file entries");
    for _ in 0..file_count {
        trace!("reading file entry name");
        let file_name = read_csharp_string(&mut reader)?;
        trace!("File name: {}", file_name);

        trace!("reading uncompressed length");
        let uncompressed_len = reader.read_i32()?;
        trace!("Uncompressed length: {}", uncompressed_len);

        trace!("reading compressed length");
        let compressed_len = reader.read_i32()?;
        trace!("Compressed length: {}", compressed_len);

        file_entries.push(ModFile {
            name: file_name,
            uncompressed_len,
            compressed_len,
        });
        #[cfg(target_os = "linux")]
        inc_progress_bar();
    }

    #[cfg(target_os = "linux")]
    {
        print_progress_bar_info("Success", "reading file entries", Color::Green, Style::Bold);
        set_progress_bar_action("Extracting", Color::Blue, Style::Bold);
    }

    assert_eq!(file_entries.len(), file_count as usize);

    info!("Extracting files");
    for file in file_entries {
        trace!("extracting file: {}", file.name);
        let mut file_data = reader.read_bytes(file.compressed_len as usize)?;

        if file.compressed_len != file.uncompressed_len {
            trace!("decompressing file: {}", file.name);
            let mut decoder = flate2::read::DeflateDecoder::new(&file_data[..]);
            let mut uncompressed_data = Vec::with_capacity(file.uncompressed_len as usize);
            decoder.read_to_end(&mut uncompressed_data)?;
            file_data = uncompressed_data;
        } else {
            trace!("file is not compressed: {}", file.name);
        }
        let file_path = out_dir.join(&file.name);
        if let Some(parent) = file_path.parent() {
            trace!("checking if file's parent directory exists: {:?}", parent);
            if !parent.try_exists()? {
                trace!("creating parent directory: {:?}", parent);
                std::fs::create_dir_all(parent)?;
            }
        }
        trace!("writing file: {:?}", file_path);
        std::fs::write(file_path, file_data)?;

        #[cfg(target_os = "linux")]
        inc_progress_bar();
    }

    #[cfg(target_os = "linux")]
    {
        print_progress_bar_info("Success", "extracting files", Color::Green, Style::Bold);
        finalize_progress_bar();
    }

    info!("Done! Your files are in: {:?}", out_dir);

    Ok(())
}

// read a 7 bit encoded string length
// then read that many bytes into a string
fn read_csharp_string(reader: &mut BinaryReader) -> Result<String, TModError> {
    let mut string_len = 0;
    let mut done = false;
    let mut step = 0;
    while !done {
        let byte = reader.read_u8()?;
        string_len |= ((byte & 0x7F) as u32) << (step * 7);
        done = (byte & 0x80) == 0;
        step += 1;
    }
    let buf = reader.read_bytes(string_len as usize)?;
    Ok(String::from_utf8(buf)?)
}
