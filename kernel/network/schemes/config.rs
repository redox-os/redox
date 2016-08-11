use alloc::boxed::Box;
use fs::{KScheme, Resource, SliceResource, SliceMutResource};
use network::common::{DNS_ADDR, IP_ADDR, IP_ROUTER_ADDR, IP_SUBNET, MAC_ADDR};
use system::error::{Error, ENOENT, Result};
use system::syscall::{MODE_DIR, MODE_FILE};

/// Network configuration scheme
pub struct NetConfigScheme;

impl KScheme for NetConfigScheme {
    fn scheme(&self) -> &str {
        "netcfg"
    }

    fn open(&mut self, url: &str, _: usize) -> Result<Box<Resource>> {
        match url.splitn(2, ":").nth(1).unwrap_or("") {
            "dns" => Ok(Box::new(SliceMutResource::new("netcfg:dns", unsafe { &mut DNS_ADDR.bytes }, MODE_FILE))),
            "ip" => Ok(Box::new(SliceMutResource::new("netcfg:ip", unsafe { &mut IP_ADDR.bytes }, MODE_FILE))),
            "ip_router" => Ok(Box::new(SliceMutResource::new("netcfg:ip_router", unsafe { &mut IP_ROUTER_ADDR.bytes }, MODE_FILE))),
            "ip_subnet" => Ok(Box::new(SliceMutResource::new("netcfg:ip_subnet", unsafe { &mut IP_SUBNET.bytes }, MODE_FILE))),
            "mac" => Ok(Box::new(SliceMutResource::new("netcfg:mac", unsafe { &mut MAC_ADDR.bytes }, MODE_FILE))),
            "" => Ok(Box::new(SliceResource::new("netcfg:", b"dns\nip\nmac", MODE_DIR))),
            _ => Err(Error::new(ENOENT))
        }
    }
}
