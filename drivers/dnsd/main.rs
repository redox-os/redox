use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::time;

use dns::{Dns, DnsQuery};

mod dns;

fn main(){
    let mut dns = [0; 4];
    File::open("netcfg:dns").unwrap().read(&mut dns).unwrap();

    println!("DNS: Server: {:?}", dns);

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

    let mut socket = File::open(&format!("udp:{}.{}.{}.{}:53", dns[0], dns[1], dns[2], dns[3])).unwrap();
    let _sent = socket.write(&packet_data).unwrap();

    socket.flush().unwrap();

    let mut buf = [0; 65536];
    let count = socket.read(&mut buf).unwrap();

    match Dns::parse(&buf[.. count]) {
        Ok(response) => for answer in response.answers.iter() {
            println!("DNS {} {:?}", count, answer.data);
        },
        Err(err) => {
            println!("DNS {} {}", count, err);
        }
    }
}
