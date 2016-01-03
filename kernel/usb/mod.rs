use self::setup::Setup;

pub mod desc;
pub mod ehci;
pub mod ohci;
pub mod setup;
pub mod uhci;
pub mod xhci;

#[derive(Debug)]
pub enum UsbMsg<'a> {
    Setup(&'a Setup),
    In(&'a mut [u8]),
    Out(&'a [u8]),
}
