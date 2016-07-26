use std::fs::File;
use std::io::{Read, Write};
use std::time;

use dhcp::Dhcp;

mod dhcp;

fn main(){
    {
        let mut current_ip = [0; 4];
        File::open("netcfg:ip").unwrap().read(&mut current_ip).unwrap();

        println!("DHCP: Current IP: {:?}", current_ip);
    }

    let tid = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().subsec_nanos();

    let packet = Dhcp {
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
        chaddr: [0x52, 0x54, 0x00, 0x12, 0x34, 0x56, 0x00, 0x00,
                 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        sname: [0; 64],
        file: [0; 128],
        magic: 0x63825363u32.to_be(),
        options: [53, 1, 1, 255]
    };

    let packet_data = unsafe { std::slice::from_raw_parts((&packet as *const Dhcp) as *const u8, std::mem::size_of::<Dhcp>()) };

    let mut socket = File::open("udp:255.255.255.255:67/68").unwrap();
    let _sent = socket.write(packet_data).unwrap();

    socket.flush().unwrap();

    let mut buf = [0; 65536];
    socket.read(&mut buf).unwrap();

    let response = unsafe { &* (buf.as_ptr() as *const Dhcp) };

    println!("DHCP: Suggested IP: {:?}, Server IP: {:?}", response.yiaddr, response.siaddr);

    {
        File::open("netcfg:ip").unwrap().write(&response.yiaddr).unwrap();
    }

    {
        let mut new_ip = [0; 4];
        File::open("netcfg:ip").unwrap().read(&mut new_ip).unwrap();

        println!("DHCP: New IP: {:?}", new_ip);
    }
}
