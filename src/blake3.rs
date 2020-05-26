use blake3::Hasher;
use std::{
    fs,
    io::{Read, Result},
    path::Path,
    time::Duration,
};

use crate::progress_bar::{ProgressBar, ProgressBarRead};

pub fn blake3<R: Read>(r: &mut R) -> Result<String> {
    let mut hasher = Hasher::new();

    let mut data = vec![0; 4 * 1024 * 1024];
    loop {
        let count = r.read(&mut data)?;
        if count == 0 {
            break;
        }

        hasher.update(&data[..count]);
    }

    let hash = hasher.finalize();
    Ok(format!("{}", hash.to_hex()))
}

pub fn blake3_progress<P: AsRef<Path>>(path: P) -> Result<String> {
    let len = fs::metadata(&path)?.len();

    let mut f = fs::File::open(&path)?;

    let mut pb = ProgressBar::new(len);
    pb.message("blake3: ");
    pb.set_max_refresh_rate(Some(Duration::new(1, 0)));
    pb.set_units(pbr::Units::Bytes);

    let res = {
        let mut pbr = ProgressBarRead::new(&mut pb, &mut f);
        blake3(&mut pbr)
    };

    pb.finish_println("");

    res
}
