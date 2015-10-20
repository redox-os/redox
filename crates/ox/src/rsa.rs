//! This is a temporary implementation of rsa until liboctavo is ported to redox
//! Note that this is not secure. It's trivial to crack. This implementation is just
//! for testing.

fn mod_pow(b: u64, e: u64, m: u64) -> u64 {
    let mut c = 1;
    let mut e_prime = 0;

    loop {
        e_prime += 1;
        c = (b * c) % m;

        if e_prime >= e {
            break;
        }
    }

    c
}

fn encrypt(msg: u64, key: (u64, u64)) -> u64 {
    mod_pow(msg, key.1, key.0)
}
fn decrypt(enc_msg: u64, key: (u64, u64)) -> u64 {
    mod_pow(enc_msg, key.1, key.0)
}
