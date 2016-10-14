use collections::Vec;

use scheme;
use syscall::error::Result;

pub fn resource() -> Result<Vec<u8>> {
    let mut data = Vec::new();

    let schemes = scheme::schemes();
    for (name, _scheme_lock) in schemes.iter_name() {
        data.extend_from_slice(name);
        data.push(b'\n');
    }

    Ok(data)
}
