use cell::UnsafeCell;
use fs::File;
use io::{Error, ErrorKind, Result, Read, Write};
use iter::Iterator;
use net::{Ipv4Addr, SocketAddr, SocketAddrV4, Shutdown};
use string::ToString;
use system::error::EINVAL;
use time::{self, Duration};
use vec::{IntoIter, Vec};

use super::dns::{Dns, DnsQuery};

pub struct LookupHost(IntoIter<SocketAddr>);

impl Iterator for LookupHost {
    type Item = SocketAddr;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub fn lookup_host(host: &str) -> Result<LookupHost> {
    let mut dns = [0; 4];
    try!(try!(File::open("netcfg:dns")).read(&mut dns));

    let tid = (time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().subsec_nanos() >> 16) as u16;

    let packet = Dns {
        transaction_id: tid,
        flags: 0x0100,
        queries: vec![DnsQuery {
            name: host.to_string(),
            q_type: 0x0001,
            q_class: 0x0001,
        }],
        answers: vec![]
    };

    let packet_data = packet.compile();

    let mut socket = try!(File::open(&format!("udp:{}.{}.{}.{}:53", dns[0], dns[1], dns[2], dns[3])));
    try!(socket.write(&packet_data));
    try!(socket.flush());

    let mut buf = [0; 65536];
    let count = try!(socket.read(&mut buf));

    match Dns::parse(&buf[.. count]) {
        Ok(response) => {
            let mut addrs = vec![];
            for answer in response.answers.iter() {
                if answer.a_type == 0x0001 && answer.a_class == 0x0001 && answer.data.len() == 4 {
                    let addr = Ipv4Addr::new(answer.data[0], answer.data[1], answer.data[2], answer.data[3]);
                    addrs.push(SocketAddr::V4(SocketAddrV4::new(addr, 0)));
                }
            }
            Ok(LookupHost(addrs.into_iter()))
        },
        Err(_err) => Err(Error::new_sys(EINVAL))
    }
}

#[derive(Debug)]
pub struct TcpStream(UnsafeCell<File>);

impl TcpStream {
    pub fn connect(addr: &SocketAddr) -> Result<TcpStream> {
        let path = format!("tcp:{}", addr);
        Ok(TcpStream(UnsafeCell::new(try!(File::open(path)))))
    }

    pub fn duplicate(&self) -> Result<TcpStream> {
        unsafe { (*self.0.get()).dup().map(|file| TcpStream(UnsafeCell::new(file))) }
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        unsafe { (*self.0.get()).read(buf) }
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> Result<usize> {
        unsafe { (*self.0.get()).read_to_end(buf) }
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        unsafe { (*self.0.get()).write(buf) }
    }

    pub fn take_error(&self) -> Result<Option<Error>> {
        Ok(None)
    }

    pub fn peer_addr(&self) -> Result<SocketAddr> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn socket_addr(&self) -> Result<SocketAddr> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn shutdown(&self, _how: Shutdown) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn nodelay(&self) -> Result<bool> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn nonblocking(&self) -> Result<bool> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn only_v6(&self) -> Result<bool> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn ttl(&self) -> Result<u32> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn read_timeout(&self) -> Result<Option<Duration>> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn write_timeout(&self) -> Result<Option<Duration>> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_nodelay(&self, _nodelay: bool) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_nonblocking(&self, _nonblocking: bool) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_only_v6(&self, _only_v6: bool) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_ttl(&self, _ttl: u32) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_read_timeout(&self, _dur: Option<Duration>) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_write_timeout(&self, _dur: Option<Duration>) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
}

#[derive(Debug)]
pub struct TcpListener(File);

impl TcpListener {
    pub fn bind(_addr: &SocketAddr) -> Result<TcpListener> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn duplicate(&self) -> Result<TcpListener> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn take_error(&self) -> Result<Option<Error>> {
        Ok(None)
    }

    pub fn socket_addr(&self) -> Result<SocketAddr> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn nonblocking(&self) -> Result<bool> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn only_v6(&self) -> Result<bool> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn ttl(&self) -> Result<u32> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_nonblocking(&self, _nonblocking: bool) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_only_v6(&self, _only_v6: bool) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_ttl(&self, _ttl: u32) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
}

#[derive(Debug)]
pub struct UdpSocket(UnsafeCell<File>);

impl UdpSocket {
    pub fn bind(addr: &SocketAddr) -> Result<UdpSocket> {
        let path = format!("udp:{}", addr);
        Ok(UdpSocket(UnsafeCell::new(try!(File::open(path)))))
    }

    pub fn duplicate(&self) -> Result<UdpSocket> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        unsafe { (*self.0.get()).read(buf) }
    }

    pub fn send(&self, buf: &[u8]) -> Result<usize> {
        unsafe { (*self.0.get()).write(buf) }
    }

    pub fn take_error(&self) -> Result<Option<Error>> {
        Ok(None)
    }

    pub fn socket_addr(&self) -> Result<SocketAddr> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn broadcast(&self) -> Result<bool> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn nonblocking(&self) -> Result<bool> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn only_v6(&self) -> Result<bool> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn ttl(&self) -> Result<u32> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn read_timeout(&self) -> Result<Option<Duration>> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn write_timeout(&self) -> Result<Option<Duration>> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_broadcast(&self, _broadcast: bool) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_nonblocking(&self, _nonblocking: bool) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_only_v6(&self, _only_v6: bool) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_ttl(&self, _ttl: u32) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_read_timeout(&self, _dur: Option<Duration>) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }

    pub fn set_write_timeout(&self, _dur: Option<Duration>) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    }
}
