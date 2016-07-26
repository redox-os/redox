use alloc::boxed::Box;
use fs::{KScheme, Resource, SliceResource, SliceMutResource, Url};
use network::common::{MAC_ADDR, IP_ADDR};
use system::error::{Error, ENOENT, Result};

/// Network configuration scheme
pub struct NetConfigScheme;

impl KScheme for NetConfigScheme {
    fn scheme(&self) -> &str {
        "netcfg"
    }

    fn open(&mut self, url: Url, _: usize) -> Result<Box<Resource>> {
        match url.reference() {
            "mac" => Ok(Box::new(SliceMutResource::new("netcfg:mac", unsafe { &mut MAC_ADDR.bytes }))),
            "ip" => Ok(Box::new(SliceMutResource::new("netcfg:ip", unsafe { &mut IP_ADDR.bytes }))),
            "" => Ok(Box::new(SliceResource::new("netcfg:", b"mac\nip"))),
            _ => Err(Error::new(ENOENT))
        }
    }
}
