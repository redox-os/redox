use std::rand;
use std::{str, u16};

use resource_scheme::ResourceScheme;
use syscall;
use syscall::error::{Error, Result, ENOENT, EINVAL};
use syscall::flag::O_RDWR;

use common::{Ipv4Addr, Udp};
use resource::UdpResource;

/// UDP UdpScheme
pub struct UdpScheme;

impl ResourceScheme<UdpResource> for UdpScheme {
    fn open_resource(&self, url: &[u8], _flags: usize, _uid: u32, _gid: u32) -> Result<Box<UdpResource>> {
        let path = try!(str::from_utf8(url).or(Err(Error::new(EINVAL))));
        let mut parts = path.split('/');
        let remote = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("");

        // Check host and port vs path
        if remote.is_empty() {
            let host_port = path.parse::<u16>().unwrap_or(0);
            if host_port > 0 {
                while let Ok(ip) = syscall::open("ip:/11", O_RDWR) {
                    let mut bytes = [0; 65536];
                    if let Ok(count) = syscall::read(ip, &mut bytes) {
                        if let Some(datagram) = Udp::from_bytes(&bytes[..count]) {
                            if datagram.header.dst.get() == host_port {
                                let mut path = [0; 256];
                                if let Ok(path_count) = syscall::fpath(ip, &mut path) {
                                    let ip_reference = unsafe { str::from_utf8_unchecked(&path[.. path_count]) }.split(':').nth(1).unwrap_or("");
                                    let peer_addr = ip_reference.split('/').next().unwrap_or("").split(':').next().unwrap_or("");

                                    return Ok(Box::new(UdpResource {
                                        ip: ip,
                                        data: datagram.data,
                                        peer_addr: Ipv4Addr::from_str(peer_addr),
                                        peer_port: datagram.header.src.get(),
                                        host_port: host_port,
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        } else {
            let mut remote_parts = remote.split(':');
            let peer_addr = remote_parts.next().unwrap_or("");
            let peer_port = remote_parts.next().unwrap_or("").parse::<u16>().unwrap_or(0);
            if peer_port > 0 {
                let host_port = path.parse::<u16>().unwrap_or((rand() % 32768 + 32768) as u16);
                if let Ok(ip) = syscall::open(&format!("ip:{}/11", peer_addr), O_RDWR) {
                    return Ok(Box::new(UdpResource {
                        ip: ip,
                        data: Vec::new(),
                        peer_addr: Ipv4Addr::from_str(peer_addr),
                        peer_port: peer_port as u16,
                        host_port: host_port,
                    }));
                }
            }
        }

        Err(Error::new(ENOENT))
    }
}
