use blake3::Hasher;
use std::{fs, io::Result, path::Path, time::Duration};

use crate::progress_bar::{ProgressBar, ProgressBarRead};

pub fn blake3_progress<P: AsRef<Path>>(path: P) -> Result<String> {
    let len = fs::metadata(&path)?.len();

    let mut f = fs::File::open(&path)?;

    let mut pb = ProgressBar::new(len);
    pb.message("blake3: ");
    pb.set_max_refresh_rate(Some(Duration::new(1, 0)));
    pb.set_units(pbr::Units::Bytes);

    let mut pbr = ProgressBarRead::new(&mut pb, &mut f);
    let hash = Hasher::new().update_reader(&mut pbr)?.finalize();
    let res = format!("{}", hash.to_hex());

    pb.finish_println("");

    Ok(res)
}

pub fn blake3_silent<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut f = fs::File::open(&path)?;

    let hash = Hasher::new().update_reader(&mut f)?.finalize();
    let res = format!("{}", hash.to_hex());
    Ok(res)
}

pub fn hash_to_hex(h: [u8; 32]) -> String {
    format!("{}", blake3::Hash::from_bytes(h).to_hex())
}
