use std::{
    fs,
    io::{Read, Result},
    path::Path,
    time::Duration,
};
use sha2::{Digest, Sha256};

use crate::progress_bar::{ProgressBar, ProgressBarRead};

pub fn sha256<R: Read>(r: &mut R) -> Result<String> {
    let mut hasher = Sha256::default();

    let mut data = vec![0; 4 * 1024 * 1024];
    loop {
        let count = r.read(&mut data)?;
        if count == 0 {
            break;
        }

        hasher.input(&data[..count]);
    }

    Ok(format!("{:x}", hasher.result()))
}

pub fn sha256_progress<P: AsRef<Path>>(path: P) -> Result<String> {
    let len = fs::metadata(&path)?.len();

    let mut f = fs::File::open(&path)?;

    let mut pb = ProgressBar::new(len);
    pb.message("sha256: ");
    pb.set_max_refresh_rate(Some(Duration::new(1, 0)));
    pb.set_units(pbr::Units::Bytes);

    let res = {
        let mut pbr = ProgressBarRead::new(&mut pb, &mut f);
        sha256(&mut pbr)
    };

    pb.finish_println("");

    res
}
