use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

use crate::errors::Result;

pub async fn sha256_file(file: PathBuf) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut file = tokio::fs::File::open(file).await?;
    let mut buffer = [0; 10240];
    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
