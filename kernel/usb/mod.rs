pub use self::hci::Hci;
pub use self::setup::Setup;

pub mod desc;
pub mod ehci;
pub mod hci;
pub mod ohci;
pub mod setup;
pub mod uhci;
pub mod xhci;

#[derive(Debug)]
pub enum Packet<'a> {
    Setup(&'a Setup),
    In(&'a mut [u8]),
    Out(&'a [u8]),
}

#[derive(Debug)]
pub enum Pipe {
    Control,
    Interrupt,
    Isochronous,
    Bulk,
}
