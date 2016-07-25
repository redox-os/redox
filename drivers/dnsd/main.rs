use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::time;

use dns::{Dns, DnsQuery};

mod dns;

fn main(){
    let tid = (time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().subsec_nanos() >> 16) as u16;

    let packet = Dns {
        transaction_id: tid,
        flags: 0x0100,
        queries: vec![DnsQuery {
            name: env::args().nth(1).unwrap_or("static.redox-os.org".to_string()),
            q_type: 0x0001,
            q_class: 0x0001,
        }],
        answers: vec![]
    };

    let packet_data = packet.compile();

    let mut socket = File::open("udp:10.0.2.3:53").unwrap();
    let _sent = socket.write(&packet_data).unwrap();

    socket.flush().unwrap();

    let mut buf = [0; 65536];
    let count = socket.read(&mut buf).unwrap();

    match Dns::parse(&buf[.. count]) {
        Ok(response) => {
            println!("DNS {} {:#?}", count, response);
        },
        Err(err) => {
            println!("DNS {} {}", count ,err);
        }
    }
}
