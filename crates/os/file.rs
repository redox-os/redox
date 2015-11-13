pub struct File<'a> {
    name: &'a [u8],
    content: &'a [u8],
}

impl File {
    pub fn from_bytes(b: &[u8]) -> Self {
        // Clear the padding
        let (mut l, mut b) = b.split_last();
        while l == 0 {
            let split = b.split_last();
            l = split.0;
            b = split.1;
        }


        // The header is 128 bytes
        // Extract the file name
        let title = &b[..32];
        // Get the file
        let file = &b[128..];

        File {
            name: title,
            content: file,
        }

    }
}
