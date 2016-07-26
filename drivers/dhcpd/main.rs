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
        let discover = Dhcp {
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
            options: [53, 1, 1, 255, 0, 0, 0, 0, 0, 0]
        };

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
        let request = Dhcp {
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
            options: [53, 1, 3, 50, 4, offer.yiaddr[0], offer.yiaddr[1], offer.yiaddr[2], offer.yiaddr[3], 255]
        };

        let request_data = unsafe { std::slice::from_raw_parts((&request as *const Dhcp) as *const u8, std::mem::size_of::<Dhcp>()) };

        let _sent = socket.write(request_data).unwrap();
        socket.flush().unwrap();

        println!("DHCP: Sent Request");
    }

    let mut ack_data = [0; 65536];
    socket.read(&mut ack_data).unwrap();
    let ack = unsafe { &* (ack_data.as_ptr() as *const Dhcp) };
    println!("DHCP: Ack IP: {:?}, Server IP: {:?}", ack.yiaddr, ack.siaddr);

    {
        File::open("netcfg:ip").unwrap().write(&ack.yiaddr).unwrap();

        let mut new_ip = [0; 4];
        File::open("netcfg:ip").unwrap().read(&mut new_ip).unwrap();

        println!("DHCP: New IP: {:?}", new_ip);
    }
}
