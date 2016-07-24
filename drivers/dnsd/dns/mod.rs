#[repr(packed)]
pub struct Dns {
    pub transaction_id: u16,
    pub flags: u16,
    pub questions: u16,
    pub answers: u16,
    pub authorities: u16,
    pub additional: u16,
    pub req: [u8; 18],
    pub req_type: u16,
    pub req_class: u16
}
