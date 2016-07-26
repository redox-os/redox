use std::fs::File;
use std::io::{Read, Write};
use std::time;

use dhcp::Dhcp;

mod dhcp;

fn main(){
    let mut current_mac = [0; 6];
    File::open("netcfg:mac").unwrap().read(&mut current_mac).unwrap();

    {
        let mut current_ip = [0; 4];
        File::open("netcfg:ip").unwrap().read(&mut current_ip).unwrap();

        println!("DHCP: Current IP: {:?}", current_ip);
    }

    let tid = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().subsec_nanos();

    let mut socket = File::open("udp:255.255.255.255:67/68").unwrap();

    {
        let mut discover = Dhcp {
            op: 1,
            htype: 1,
            hlen: 6,
            hops: 0,
            tid: tid,
            secs: 0,
            flags: 0x8000u16.to_be(),
            ciaddr: [0, 0, 0, 0],
            yiaddr: [0, 0, 0, 0],
            siaddr: [0, 0, 0, 0],
            giaddr: [0, 0, 0, 0],
            chaddr: [current_mac[0], current_mac[1], current_mac[2], current_mac[3], current_mac[4], current_mac[5],
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            sname: [0; 64],
            file: [0; 128],
            magic: 0x63825363u32.to_be(),
            options: [0; 308]
        };

        for (s, mut d) in [53, 1, 1, 255].iter().zip(discover.options.iter_mut()) {
            *d = *s;
        }

        let discover_data = unsafe { std::slice::from_raw_parts((&discover as *const Dhcp) as *const u8, std::mem::size_of::<Dhcp>()) };

        let _sent = socket.write(discover_data).unwrap();
        socket.flush().unwrap();

        println!("DHCP: Sent Discover");
    }

    let mut offer_data = [0; 65536];
    socket.read(&mut offer_data).unwrap();
    let offer = unsafe { &* (offer_data.as_ptr() as *const Dhcp) };
    println!("DHCP: Offer IP: {:?}, Server IP: {:?}", offer.yiaddr, offer.siaddr);

    {
        let mut request = Dhcp {
            op: 1,
            htype: 1,
            hlen: 6,
            hops: 0,
            tid: tid,
            secs: 0,
            flags: 0,
            ciaddr: [0; 4],
            yiaddr: [0; 4],
            siaddr: offer.siaddr,
            giaddr: [0; 4],
            chaddr: [current_mac[0], current_mac[1], current_mac[2], current_mac[3], current_mac[4], current_mac[5],
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            sname: [0; 64],
            file: [0; 128],
            magic: 0x63825363u32.to_be(),
            options: [0; 308]
        };

        for (s, mut d) in [53, 1, 3, 50, 4, offer.yiaddr[0], offer.yiaddr[1], offer.yiaddr[2], offer.yiaddr[3], 255].iter().zip(request.options.iter_mut()) {
            *d = *s;
        }

        let request_data = unsafe { std::slice::from_raw_parts((&request as *const Dhcp) as *const u8, std::mem::size_of::<Dhcp>()) };

        let _sent = socket.write(request_data).unwrap();
        socket.flush().unwrap();

        println!("DHCP: Sent Request");
    }

    let mut ack_data = [0; 65536];
    socket.read(&mut ack_data).unwrap();
    let ack = unsafe { &* (ack_data.as_ptr() as *const Dhcp) };
    println!("DHCP: Ack IP: {:?}, Server IP: {:?}", ack.yiaddr, ack.siaddr);

    let mut subnet_option = None;
    let mut gateway_option = None;
    let mut dns_option = None;

    let mut options = ack.options.iter();
    while let Some(option) = options.next() {
        match *option {
            0 => (),
            255 => break,
            _ => if let Some(len) = options.next() {
                if *len as usize <= options.as_slice().len() {
                    let data = &options.as_slice()[.. *len as usize];
                    for _data_i in 0..*len {
                        options.next();
                    }
                    match *option {
                        1 => {
                            println!("DHCP: Subnet Mask: {:?}", data);
                            if data.len() == 4 && subnet_option.is_none() {
                                subnet_option = Some(Vec::from(data));
                            }
                        },
                        3 => {
                            println!("DHCP: Router: {:?}", data);
                            if data.len() == 4  && gateway_option.is_none() {
                                gateway_option = Some(Vec::from(data));
                            }
                        },
                        6 => {
                            println!("DHCP: Domain Name Server: {:?}", data);
                            if data.len() == 4 && dns_option.is_none() {
                                dns_option = Some(Vec::from(data));
                            }
                        },
                        51 => println!("DHCP: Lease Time: {:?}", data),
                        53 => println!("DHCP: Message Type: {:?}", data),
                        54 => println!("DHCP: Server ID: {:?}", data),
                        _ => println!("DHCP: {}: {:?}", option, data)
                    }
                }
            },
        }
    }

    {
        File::open("netcfg:ip").unwrap().write(&ack.yiaddr).unwrap();

        let mut new_ip = [0; 4];
        File::open("netcfg:ip").unwrap().read(&mut new_ip).unwrap();

        println!("DHCP: New IP: {:?}", new_ip);
    }

    if let Some(dns) = dns_option {
        File::open("netcfg:dns").unwrap().write(&dns).unwrap();

        let mut new_dns = [0; 4];
        File::open("netcfg:dns").unwrap().read(&mut new_dns).unwrap();

        println!("DHCP: New DNS: {:?}", new_dns);
    }
}
