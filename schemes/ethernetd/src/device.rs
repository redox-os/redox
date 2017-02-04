use std::fs::File;
use smoltcp;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt;


const IPV4_BROADCAST: smoltcp::wire::IpAddress = smoltcp::wire::IpAddress::Ipv4(smoltcp::wire::Ipv4Address([255,255,255,255]));
const ETHERNET_BROADCAST: smoltcp::wire::EthernetAddress = smoltcp::wire::EthernetAddress([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);

// FIXME: we should have one network: resource per physical interface. Currently it's ok because we
// support only one physical interface. However, what happens when a broadcast frame arrived on the
// network scheme, and we have multiple interfaces? We have no way to tell for which interface it
// is.
struct Device {
    pub network: File,
}

struct TxBuffer {
    buffer: Vec<u8>,
    network: File,
}

impl AsRef<[u8]> for TxBuffer {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}

impl AsMut<[u8]> for TxBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        self.buffer.as_mut()
    }
}

impl Drop for TxBuffer {
    fn drop(&mut self) {
        self.network.write(&mut self.buffer[..]).unwrap();
    }
}

// FIXME: with this implementation
//  - we allocate a new buffer every time we receive or send a frame. We should reuse buffers as
//    much a possible
//  - we perform a copy for each frame we send and receive. This can only be fixed if the driver
//    implemented the Device trait, afaict.
impl smoltcp::phy::Device for Device {
    type RxBuffer = Vec<u8>;
    type TxBuffer = TxBuffer;

    fn mtu(&self) -> usize {
        1536
    }

    fn receive(&mut self) -> Result<Self::RxBuffer, smoltcp::Error> {
        let mut buffer = vec![0; self.mtu()];
        let size = self.network.read(&mut buffer[..]).unwrap();
        buffer.resize(size, 0);
        Ok(buffer)
    }

    fn transmit(&mut self, length: usize) -> Result<Self::TxBuffer, smoltcp::Error> {
        Ok(TxBuffer {
            network:  self.network.try_clone().unwrap(),
            buffer: vec![0; length]
        })
    }
}

pub struct EthernetDevice(smoltcp::iface::EthernetInterface<'static, 'static, 'static, Device>);

impl EthernetDevice {
    pub fn new(network: File) -> Self {
        let device = Box::new(Device { network: network });
        let mut arp_cache = Box::new(smoltcp::iface::SliceArpCache::new(vec![Default::default(); 8])) as Box<smoltcp::iface::ArpCache>;
        arp_cache.fill(&IPV4_BROADCAST, &ETHERNET_BROADCAST);

        // FIXME: I don't know where/when/how the MAC address should be set.
        let hardware_addr = smoltcp::wire::EthernetAddress([0x0, 0x0, 0x10, 0x10, 0x10, 0x10]);
        EthernetDevice(smoltcp::iface::EthernetInterface::new(device, arp_cache, hardware_addr, vec![]))
    }

    pub fn set_mac_address(&mut self, mac_address: &str) -> Result<(), ()> {
        let addr = smoltcp::wire::EthernetAddress::parse(mac_address).or(Err(()))?;
        self.0.set_hardware_addr(addr);
        Ok(())
    }

	pub fn poll(&mut self, sockets: &mut smoltcp::socket::SocketSet) -> Result<(), ()> {
		let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() * 1000;
		self.0.poll(sockets, timestamp).or_else(|e| {
            match e {
                smoltcp::Error::Truncated => {
                    // We ignore this one because it's returned when there is no new packet to
                    // read, which is not really an error.
                    Ok(())
				},
                _ => {
                    println!("ethernet device: Error while polling: {}", e);
                    Err(())
                },
			}
		})
	}

    pub fn set_ipv4_address(&mut self, addr: &str) -> Result<(), ()> {
        let ipv4_addr = smoltcp::wire::IpAddress::Ipv4(smoltcp::wire::Ipv4Address::parse(addr)?);
        self.0.update_protocol_addrs(|addrs| {
            let mut new_addrs = addrs.to_vec();
            for addr in new_addrs.iter() {
                if *addr == ipv4_addr {
                    return;
                }
            }
            new_addrs.push(ipv4_addr);
            *addrs = new_addrs.into();
        });
        Ok(())
    }

    pub fn del_ipv4_address(&mut self, addr: &str) -> Result<(), ()> {
        let addr_to_delete = smoltcp::wire::IpAddress::Ipv4(smoltcp::wire::Ipv4Address::parse(addr)?);
        self.0.update_protocol_addrs(|addrs| {
            let mut new_addrs: Vec<smoltcp::wire::IpAddress>  = Vec::with_capacity(addrs.len());
            let mut old_addrs = addrs.to_vec();
            for addr in old_addrs.drain(..) {
                if addr != addr_to_delete {
                    new_addrs.push(addr);
                }
            }
            *addrs = new_addrs.into();
        });
        Ok(())
    }
}

impl fmt::Display for EthernetDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0.hardware_addr()).unwrap();
        for addr in self.0.protocol_addrs() {
            write!(f, ",{}", addr).unwrap();
        }
        Ok(())
    }
}

impl fmt::Debug for EthernetDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "EthernetDevice({:?}", self.0.hardware_addr()).unwrap();
        for addr in self.0.protocol_addrs() {
            write!(f, ",{:?}", addr).unwrap();
        }
        write!(f, ")").unwrap();
        Ok(())
    }
}
