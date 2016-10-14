use collections::Vec;

use scheme;
use syscall::error::Result;

pub fn resource() -> Result<Vec<u8>> {
    let mut data = Vec::new();

    let schemes = scheme::schemes();
    for (name, _scheme_lock) in schemes.iter_name() {
        if ! data.is_empty() {
            data.push(b'\n');
        }
        data.extend_from_slice(name);
    }

    Ok(data)
}
