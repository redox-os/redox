use std::collections::BTreeMap;
use std::fs::File;
use std::{str, u16};

use syscall::error::{EACCES, EBADF, EINVAL, EIO, EWOULDBLOCK, Result, Error};
use syscall::flag::O_NONBLOCK;
use syscall::scheme::SchemeMut;

use smoltcp::socket::{SocketSet, SocketHandle, UdpSocket, UdpSocketBuffer, UdpPacketBuffer, AsSocket};
use smoltcp::wire::{Ipv4Address, IpEndpoint, IpAddress};
use device::EthernetDevice;
use rand;
use rand::Rng;

const DUMMY_HANDLE: usize = 0;
const DEVICE_HANDLE: usize = 1;
// const ETHERNET_SOCKET_HANDLE: usize = 2;
// const IP_SOCKET_HANDLE: usize = 3;
// const ICMP_SOCKET_HANDLE: usize = 4;

// (ethernet mtu) - (max ip header len) - (max udp header len) = 1536 - 60 - 8 = 1468
const UDP_PAYLOAD: usize = 1468;

struct UdpHandle {
    /// The smoltcp handle, used to retrieve the socket from the sockets set.
    handle: SocketHandle,
    flags: usize,

    connected: bool,

    peer_from: Option<IpEndpoint>,
    peer_to: Option<IpEndpoint>,
}

impl UdpHandle {
    fn new(handle: SocketHandle, flags: usize) -> Self {
        UdpHandle {
            handle: handle,
            flags: flags,

            connected: false,
            peer_from: None,
            peer_to: None,
        }
    }
}


struct TcpHandle {
    /// The smoltcp handle, used to retrieve the socket from the sockets set.
    handle: SocketHandle,
    flags: usize,
}

/// Handles are used to manage the set of open sockets. Each handle is associated to a socket and
/// holds information related to it.
enum Handle {
    Tcp(TcpHandle),
    Udp(UdpHandle),
    Device,
}

pub struct EthernetScheme {
    /// A representation of the ethernet device. It sends and receives packets.
    device: EthernetDevice,

    /// The set of TCP and UDP sockets. The ethernet device regularly process this set to send
    /// outgoing packets and dispatch received packets to the appropriate sockets.
    sockets: SocketSet<'static, 'static, 'static>,

    handles: BTreeMap<usize, Handle>,

    /// Keep track of local UDP ports that are in use.
    udp_ports: BTreeMap<u16, usize>,

    // /// Keep track of local TCP ports that are in use.
    // tcp_ports: BTreeMap<u16, usize>,

    /// A random number generator used to generate port numbers.
    rng: rand::OsRng,
}

impl EthernetScheme {
    pub fn new(network: File) -> Self {
        let mut handles = BTreeMap::new();
        handles.insert(DEVICE_HANDLE, Handle::Device);
        EthernetScheme {
            handles: handles,
            device: EthernetDevice::new(network),
            sockets: SocketSet::new(Vec::new()),
            // tcp_ports: BTreeMap::new(),
            udp_ports: BTreeMap::new(),
            rng: rand::OsRng::new().expect("Failed to open RNG"),
        }
    }
}

/// Parse a string representing a socket endpoint, e.g. `127.0.0.1:8080`
/// For the moment, it supports only IPv4 endpoints.
fn parse_endpoint(endpoint: String) -> Result<IpEndpoint> {
    let mut parts = endpoint.split(':');
    let ip = parts.next().ok_or(Error::new(EINVAL))?;
    let port = parts.next().ok_or(Error::new(EINVAL))?;
    Ok(
        IpEndpoint::new(
            IpAddress::Ipv4(Ipv4Address::parse(&ip).or(Err(Error::new(EINVAL)))?),
            u16::from_str_radix(port, 10).or(Err(Error::new(EINVAL)))?))
}

impl EthernetScheme {
    /// Process network event. This is called whenever there is network activity (packets ready to
    /// be sent or received).
    ///
    /// At the IP level, it handles ICMP and ARP events.
    ///
    /// At the TCP/UDP level, it checks all the sockets, sends packets that are waiting to be sent,
    /// and dispatches incoming packets to the appropriate sockets
    pub fn poll(&mut self) -> Result<()> {
        return self.device.poll(&mut self.sockets).or(Err(Error::new(EIO)));
    }

    pub fn init(&mut self, hardware_addr: &str) {
        self.device.set_mac_address(hardware_addr).expect("Failed to start the nework device");
    }
}

impl EthernetScheme {

    fn get_handle_id(&self) -> usize {
        let mut handle_id: usize = 10;
        loop {
            if ! self.handles.contains_key(&handle_id) {
                break;
            }
            handle_id += 1;
        }
        return handle_id;
    }
}

impl SchemeMut for EthernetScheme {
    /// Open a new socket. For now, only TCP and UDP sockets are supported, but later, L2 and L3
    /// sockets will be supported.
    ///
    /// Examples of valid urls:
    ///
    ///     - `udp/10.0.0.1:9000`: bind a UDP socket to a local IPv4 endpoint.
    ///     - `udp/[2001:0db8:85a3::8a2e:0370:7334]:1234`: bind a UDP socket to a local IPv6 endpoint.
    ///     - `tcp/10.0.0.1:9000`: open a TCP socket that listens on a local IPv4 endpoint.
    ///     - `tcp/[2001:0db8:85a3::8a2e:0370:7334]:1234`: open a TCP socket that listens on a local IPv6 endpoint.
    fn open(&mut self, url: &[u8], flags: usize, uid: u32, _gid: u32) -> Result<usize> {
        if uid == 0 {
            let mut parts = str::from_utf8(url).or(Err(Error::new(EINVAL)))?.split('/');
            let protocol = parts.next().ok_or(Error::new(EINVAL))?;
            match protocol {
                "tcp" => {
                    unimplemented!()
                },
                "udp" => {
                    let mut endpoint = parse_endpoint(parts.next().ok_or(Error::new(EINVAL))?.to_string())?;
                    if endpoint.port == 0 {
                        endpoint.port = self.rng.gen_range(32768, 65535);
                    }
                    let udp_rx_buffer = UdpSocketBuffer::new(vec![UdpPacketBuffer::new(vec![0; UDP_PAYLOAD])]);
                    let udp_tx_buffer = UdpSocketBuffer::new(vec![UdpPacketBuffer::new(vec![0; UDP_PAYLOAD])]);
                    let mut socket = UdpSocket::new(udp_rx_buffer, udp_tx_buffer);
                    {
                        let udp_socket: &mut UdpSocket = socket.as_socket();
                        udp_socket.bind(endpoint);
                    }

                    let handle_id = self.get_handle_id();
                    let handle = UdpHandle::new(self.sockets.add(socket), flags);
                    self.handles.insert(handle_id, Handle::Udp(handle));
                    self.udp_ports.insert(endpoint.port, handle_id);
                    return Ok(handle_id);
                },
                "device" => {
                    return Ok(DEVICE_HANDLE);
                },
                _ => {
                    return Err(Error::new(EINVAL));
                },
            }
        } else {
            Err(Error::new(EACCES))
        }
    }

    fn dup(&mut self, id: usize, buf: &[u8]) -> Result<usize> {
        let command_str = str::from_utf8(buf).or(Err(Error::new(EINVAL)))?.to_string();
        let mut parts = command_str.split('/');
        let action = parts.next().ok_or(Error::new(EINVAL))?;

        match self.handles.get_mut(&id).ok_or(Error::new(EBADF))? {
            &mut Handle::Udp(ref mut handle) => {
                match action {
                    "bind" => {
                        let endpoint = parse_endpoint(parts.next().ok_or(Error::new(EINVAL))?.to_string())?;
                        let mut socket = self.sockets.get_mut(handle.handle);
                        let socket: &mut UdpSocket = socket.as_socket();

                        self.udp_ports.remove(&socket.endpoint().port);
                        socket.bind(endpoint);
                        self.udp_ports.insert(socket.endpoint().port, id);
                        Ok(DUMMY_HANDLE)
                    },
                    "connect" => {
                        let peer = parse_endpoint(parts.next().ok_or(Error::new(EINVAL))?.to_string())?;
                        handle.peer_to = Some(peer);
                        handle.peer_from = Some(peer);
                        handle.connected = true;
                        Ok(DUMMY_HANDLE)
                    },
                    "disconnect" => {
                        handle.peer_to = None;
                        handle.peer_from = None;
                        handle.connected = false;
                        Ok(DUMMY_HANDLE)
                    },
                    "peer_to" => {
                        if handle.connected {
                            return Err(Error::new(EINVAL));
                        }
                        handle.peer_to = Some(parse_endpoint(parts.next().ok_or(Error::new(EINVAL))?.to_string())?);
                        Ok(DUMMY_HANDLE)
                    },
                    "peer_from" => {
                        if handle.connected {
                            return Err(Error::new(EINVAL));
                        }
                        handle.peer_from = Some(parse_endpoint(parts.next().ok_or(Error::new(EINVAL))?.to_string())?);
                        Ok(DUMMY_HANDLE)
                    },
                    "" => {
                        Ok(id)
                    }
                    _ => {
                        return Err(Error::new(EINVAL));
                    },
                }
            },
            _ => {
                unimplemented!();
            },
        }
    }

    /// Close a socket.
    fn close(&mut self, id: usize) -> Result<usize> {
        if id == DUMMY_HANDLE || id == DEVICE_HANDLE {
            return Ok(id);
        }

        match self.handles.get_mut(&id).ok_or(Error::new(EBADF))? {
            &mut Handle::Udp(ref mut handle) => {
                let mut socket = self.sockets.remove(handle.handle);
                let socket: &mut UdpSocket = socket.as_socket();
                self.udp_ports.remove(&socket.endpoint().port);
            },
            &mut Handle::Tcp(_) => {
                unimplemented!();
            },
            _ => {},
        }

        self.handles.remove(&id).ok_or(Error::new(EBADF))?;
        Ok(id)
    }


    /// Read a datagram from a socket and return the number of bytes read.
    fn read(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        match self.handles.get_mut(&id).ok_or(Error::new(EBADF))? {
            &mut Handle::Udp(ref mut handle) => {
                let mut socket = self.sockets.get_mut(handle.handle);
                let socket: &mut UdpSocket = socket.as_socket();
                loop {
                    match socket.recv_slice(buf) {
                        Ok((count, from)) => {
                            if let Some(peer) = handle.peer_from {
                                if peer == from {
                                    return Ok(count);
                                }
                            } else {
                                handle.peer_from = Some(from);
                                return Ok(count);
                            }
                        },
                        Err(()) => {
                            if handle.flags & O_NONBLOCK == O_NONBLOCK {
                                handle.peer_from = None;
                                return Ok(0);
                            } else {
                                return Err(Error::new(EWOULDBLOCK));
                            }
                        },
                    }
                }
            },
            &mut Handle::Device => {
                let output = format!("{}", self.device);
                let bytes = output.as_bytes();
                let mut i = 0;
                while i < buf.len() && i < bytes.len() {
                    buf[i] = bytes[i];
                    i += 1;
                }
                return Ok(i);
            },
            &mut Handle::Tcp(_) => {
                unimplemented!();
            },
        }
    }

    /// Write data to a socket.
    fn write(&mut self, id: usize, buf: &[u8]) -> Result<usize> {
        match self.handles.get_mut(&id).ok_or(Error::new(EBADF))? {
            &mut Handle::Udp(ref mut handle) => {
                let mut socket = self.sockets.get_mut(handle.handle);
                let socket: &mut UdpSocket = socket.as_socket();

                if let Some(peer) = handle.peer_to {
                    Ok(socket.send_slice(buf, peer).or(Err(Error::new(EIO)))?)
                } else {
                    Err(Error::new(EINVAL))
                }
            },
            &mut Handle::Tcp(_) => {
                unimplemented!();
            },
            &mut Handle::Device => {
                let mut parts = str::from_utf8(buf).or(Err(Error::new(EINVAL)))?.split('=');
                match parts.next() {
                    Some("set_mac") => {
                        unimplemented!();
                    },
                    Some("set_ipv4") => {
                        let ip = parts.next().ok_or(Error::new(EINVAL))?;
                        self.device.set_ipv4_address(ip).or(Err(Error::new(EINVAL)))?;
                    },
                    Some("del_ipv4") => {
                        let ip = parts.next().ok_or(Error::new(EINVAL))?;
                        self.device.del_ipv4_address(ip).or(Err(Error::new(EINVAL)))?;
                    },
                    _ => {
                        return Err(Error::new(EINVAL));
                    }
                }
                Ok(0)
            }
        }
    }

    fn fpath(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let path_string: String;
        match self.handles.get_mut(&id).ok_or(Error::new(EBADF))? {
            &mut Handle::Udp(ref mut handle) => {
                if let Some(peer) = handle.peer_from {
                    path_string = format!("{}", peer);
                } else {
                    if handle.flags & O_NONBLOCK == O_NONBLOCK {
                        return Ok(0);
                    } else {
                        return Err(Error::new(EINVAL));
                    }
                }
            },
            _ => {
                unimplemented!();
            },
        }
        let path = path_string.as_bytes();
        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }
        Ok(i)
    }

    fn fsync(&mut self, id: usize) -> Result<usize> {
        // FIXME: Is this alright? I'm not sure of the semantics of `fsync` in the context of
        // sockets.
        self.poll()?;
        Ok(id)
    }
}
