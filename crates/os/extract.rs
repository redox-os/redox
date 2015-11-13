pub enum Extract {
    Dir {
        name: Vec<u8>,
        data: Vec<Extract>,
    },
    File {
        name: Vec<u8>,
        data: Vec<u8>,
    },
}
