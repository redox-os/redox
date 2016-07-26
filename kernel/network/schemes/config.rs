use alloc::boxed::Box;
use fs::{KScheme, Resource, SliceResource, SliceMutResource, Url};
use network::common::{DNS_ADDR, IP_ADDR, IP_ROUTER_ADDR, IP_SUBNET, MAC_ADDR};
use system::error::{Error, ENOENT, Result};

/// Network configuration scheme
pub struct NetConfigScheme;

impl KScheme for NetConfigScheme {
    fn scheme(&self) -> &str {
        "netcfg"
    }

    fn open(&mut self, url: Url, _: usize) -> Result<Box<Resource>> {
        match url.reference() {
            "dns" => Ok(Box::new(SliceMutResource::new("netcfg:dns", unsafe { &mut DNS_ADDR.bytes }))),
            "ip" => Ok(Box::new(SliceMutResource::new("netcfg:ip", unsafe { &mut IP_ADDR.bytes }))),
            "ip_router" => Ok(Box::new(SliceMutResource::new("netcfg:ip_router", unsafe { &mut IP_ROUTER_ADDR.bytes }))),
            "ip_subnet" => Ok(Box::new(SliceMutResource::new("netcfg:ip_subnet", unsafe { &mut IP_SUBNET.bytes }))),
            "mac" => Ok(Box::new(SliceMutResource::new("netcfg:mac", unsafe { &mut MAC_ADDR.bytes }))),
            "" => Ok(Box::new(SliceResource::new("netcfg:", b"dns\nip\nmac"))),
            _ => Err(Error::new(ENOENT))
        }
    }
}
