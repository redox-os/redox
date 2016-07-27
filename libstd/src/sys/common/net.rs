use cell::UnsafeCell;
use fs::File;
use io::{Error, ErrorKind, Result, Read, Write};
use iter::Iterator;
use net::{SocketAddr, Shutdown};
use time::Duration;
use vec::Vec;

pub struct LookupHost;

impl Iterator for LookupHost {
    type Item = Result<SocketAddr>;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub fn lookup_host(_host: &str) -> Result<LookupHost> {
    Err(Error::new(ErrorKind::Other, "Not implemented"))
}

#[derive(Debug)]
pub struct TcpStream(UnsafeCell<File>);

impl TcpStream {
    pub fn connect(addr: &SocketAddr) -> Result<TcpStream> {
        let path = format!("tcp:{}", addr);
        Ok(TcpStream(UnsafeCell::new(try!(File::open(path)))))
    }

    pub fn duplicate(&self) -> Result<TcpStream> {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
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
