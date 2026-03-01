use std::io::Read;
use std::path::Path;

use md5::{Digest, Md5};

/// Result of triple-hash computation (for DAT verification).
pub struct RomHashes {
    pub crc32: String,
    pub md5: String,
    pub sha1: String,
}

/// Hash a reader into CRC32 + MD5 + SHA1 in a single pass.
fn hash_reader(reader: &mut impl Read) -> Result<RomHashes, String> {
    use crc32fast::Hasher as Crc32Hasher;
    use sha1::Sha1;

    let mut crc_hasher = Crc32Hasher::new();
    let mut md5_hasher = Md5::new();
    let mut sha1_hasher = Sha1::new();

    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        crc_hasher.update(&buf[..n]);
        md5_hasher.update(&buf[..n]);
        sha1_hasher.update(&buf[..n]);
    }

    Ok(RomHashes {
        crc32: format!("{:08X}", crc_hasher.finalize()),
        md5: format!("{:x}", md5_hasher.finalize()),
        sha1: format!("{:x}", sha1_hasher.finalize()),
    })
}

/// Open a file (or the first entry inside a zip) and return a boxed reader.
fn open_rom_reader(path: &Path) -> Result<Box<dyn Read>, String> {
    let lower = path.to_string_lossy().to_lowercase();
    if lower.ends_with(".zip") {
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        if archive.is_empty() {
            return Err("Empty zip archive".into());
        }
        // ZipFile borrows the archive, so we read into a buffer
        let mut inner = archive.by_index(0).map_err(|e| e.to_string())?;
        let mut data = Vec::new();
        inner.read_to_end(&mut data).map_err(|e| e.to_string())?;
        Ok(Box::new(std::io::Cursor::new(data)))
    } else {
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        Ok(Box::new(file))
    }
}

/// Compute CRC32, MD5, and SHA1 in a single read pass.
/// If the file is a ZIP, hashes the first inner entry.
///
/// Must be called from a blocking context (not async).
pub fn compute_triple_hash(path: &Path) -> Result<RomHashes, String> {
    let mut reader = open_rom_reader(path)?;
    hash_reader(&mut reader)
}

/// Compute only the MD5 hash of a file (extracting from zip if needed).
///
/// Must be called from a blocking context (not async).
pub fn compute_md5(path: &Path) -> Result<String, String> {
    let mut reader = open_rom_reader(path)?;
    let mut hasher = Md5::new();
    std::io::copy(&mut reader, &mut hasher).map_err(|e| e.to_string())?;
    Ok(format!("{:x}", hasher.finalize()))
}
