use std::path::Path;

use md5::{Digest, Md5};

/// Result of triple-hash computation (for DAT verification).
pub struct RomHashes {
    pub crc32: String,
    pub md5: String,
    pub sha1: String,
}

/// Compute CRC32, MD5, and SHA1 in a single read pass.
/// If the file is a ZIP, hashes the first inner entry.
///
/// Must be called from a blocking context (not async).
pub fn compute_triple_hash(path: &Path) -> Result<RomHashes, String> {
    use crc32fast::Hasher as Crc32Hasher;
    use sha1::Sha1;
    use std::io::Read;

    let lower = path.to_string_lossy().to_lowercase();

    let mut crc_hasher = Crc32Hasher::new();
    let mut md5_hasher = Md5::new();
    let mut sha1_hasher = Sha1::new();

    if lower.ends_with(".zip") {
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        if archive.is_empty() {
            return Err("Empty zip archive".into());
        }
        let mut inner = archive.by_index(0).map_err(|e| e.to_string())?;
        let mut buf = [0u8; 8192];
        loop {
            let n = inner.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            crc_hasher.update(&buf[..n]);
            md5_hasher.update(&buf[..n]);
            sha1_hasher.update(&buf[..n]);
        }
    } else {
        let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut buf = [0u8; 8192];
        loop {
            let n = file.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            crc_hasher.update(&buf[..n]);
            md5_hasher.update(&buf[..n]);
            sha1_hasher.update(&buf[..n]);
        }
    }

    Ok(RomHashes {
        crc32: format!("{:08X}", crc_hasher.finalize()),
        md5: format!("{:x}", md5_hasher.finalize()),
        sha1: format!("{:x}", sha1_hasher.finalize()),
    })
}
