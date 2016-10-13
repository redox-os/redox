use std::cell::UnsafeCell;
use std::rand;
use std::sync::Arc;
use std::{str, u16};

use resource_scheme::ResourceScheme;
use syscall;
use syscall::error::{Error, Result, ENOENT, EINVAL};
use syscall::flag::O_RDWR;

use common::{Ipv4Addr, Tcp, TCP_SYN, TCP_ACK};
use resource::{TcpResource, TcpStream};

/// A TCP scheme
pub struct TcpScheme;

impl ResourceScheme<TcpResource> for TcpScheme {
    fn open_resource(&self, url: &[u8], _flags: usize, _uid: u32, _gid: u32) -> Result<Box<TcpResource>> {
        let path = try!(str::from_utf8(url).or(Err(Error::new(EINVAL))));
        let mut parts = path.split('/');
        let remote = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("");

        let mut remote_parts = remote.split(':');
        let host = remote_parts.next().unwrap_or("");
        let port = remote_parts.next().unwrap_or("");

        if ! host.is_empty() && ! port.is_empty() {
            let peer_addr = Ipv4Addr::from_str(host);
            let peer_port = port.parse::<u16>().unwrap_or(0);
            let host_port = (rand() % 32768 + 32768) as u16;

            match syscall::open(&format!("ip:{}/6", peer_addr.to_string()), O_RDWR) {
                Ok(ip) => {
                    let mut stream = TcpStream {
                        ip: ip,
                        peer_addr: peer_addr,
                        peer_port: peer_port,
                        host_port: host_port,
                        sequence: rand() as u32,
                        acknowledge: 0,
                        finished: false
                    };

                    if stream.client_establish() {
                        return Ok(Box::new(TcpResource {
                            stream: Arc::new(UnsafeCell::new(stream))
                        }));
                    }
                }
                Err(err) => return Err(err),
            }
        } else if ! path.is_empty() {
            let host_port = path.parse::<u16>().unwrap_or(0);

            while let Ok(ip) = syscall::open("ip:/6", O_RDWR) {
                let mut bytes = [0; 65536];
                match syscall::read(ip, &mut bytes) {
                    Ok(count) => {
                        if let Some(segment) = Tcp::from_bytes(&bytes[..count]) {
                            if segment.header.dst.get() == host_port && segment.header.flags.get() & (TCP_SYN | TCP_ACK) == TCP_SYN {
                                let mut path = [0; 256];
                                if let Ok(path_count) = syscall::fpath(ip, &mut path) {
                                    let ip_reference = unsafe { str::from_utf8_unchecked(&path[.. path_count]) }.split(':').nth(1).unwrap_or("");
                                    let ip_remote = ip_reference.split('/').next().unwrap_or("");
                                    let peer_addr = ip_remote.split(':').next().unwrap_or("");

                                    let mut stream = TcpStream {
                                        ip: ip,
                                        peer_addr: Ipv4Addr::from_str(peer_addr),
                                        peer_port: segment.header.src.get(),
                                        host_port: host_port,
                                        sequence: rand() as u32,
                                        acknowledge: segment.header.sequence.get(),
                                        finished: false
                                    };

                                    if stream.server_establish(segment) {
                                        return Ok(Box::new(TcpResource {
                                            stream: Arc::new(UnsafeCell::new(stream))
                                        }));
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => return Err(err),
                }
            }
        }

        Err(Error::new(ENOENT))
    }
}
