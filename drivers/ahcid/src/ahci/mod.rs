use io::Io;

use self::disk::Disk;
use self::hba::{HbaMem, HbaPortType};

pub mod disk;
pub mod fis;
pub mod hba;

pub fn disks(base: usize, name: &str) -> Vec<Disk> {
    unsafe { &mut *(base as *mut HbaMem) }.init();
    let pi = unsafe { &mut *(base as *mut HbaMem) }.pi.read();
    let ret: Vec<Disk> = (0..32)
          .filter(|&i| pi & 1 << i as i32 == 1 << i as i32)
          .filter_map(|i| {
              let port = &mut unsafe { &mut *(base as *mut HbaMem) }.ports[i];
              let port_type = port.probe();
              print!("{}", format!("{}-{}: {:?}\n", name, i, port_type));
              match port_type {
                  HbaPortType::SATA => {
                      match Disk::new(i, port) {
                          Ok(disk) => Some(disk),
                          Err(err) => {
                              print!("{}", format!("{}: {}\n", i, err));
                              None
                          }
                      }
                  }
                  _ => None,
              }
          })
          .collect();

    ret
}
