// ba6ce90ac85fd41a3ff1d9b203f6de3e73a6b6da

/// Get the first file of the tar string (Header + Content)
pub fn get_file(s: &[u8]) -> File {

}

pub struct File<'a> {
    file: &'a [u8],
    header: TarHeader<'a>,
}

pub struct TarHeader<'a> {
    // I really need constant sized pointers here :(
    name: &'a [u8],
    mode: &'a [u8],
    group: &'a [u8],
    user: &'a [u8],
    size: &'a [u8],
    last_mod: &'a [u8],
    checksum: &'a [u8],
    link_ind: &u8,
    link: &[u8],
    ustar_header: UStarHeader,
}
