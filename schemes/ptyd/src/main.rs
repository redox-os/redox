#![feature(arc_counts)]

extern crate syscall;

use std::collections::{BTreeMap, VecDeque};
use std::fs::File;
use std::io::{Read, Write};
use std::{str, thread};
use std::sync::{Arc, Weak, Mutex};

use syscall::data::Packet;
use syscall::error::{Error, Result, EBADF, EINVAL, ENOENT, EPIPE, EWOULDBLOCK};
use syscall::flag::O_NONBLOCK;
use syscall::scheme::SchemeMut;

pub struct PtyScheme {
    next_id: usize,
    ptys: (BTreeMap<usize, PtyMaster>, BTreeMap<usize, PtySlave>)
}

impl PtyScheme {
    fn new() -> Self {
        PtyScheme {
            next_id: 0,
            ptys: (BTreeMap::new(), BTreeMap::new())
        }
    }
}

impl SchemeMut for PtyScheme {
    fn open(&mut self, path: &[u8], flags: usize, _uid: u32, _gid: u32) -> Result<usize> {
        let path = str::from_utf8(path).or(Err(Error::new(EINVAL)))?.trim_matches('/');

        if path.is_empty() {
            let id = self.next_id;
            self.next_id += 1;

            self.ptys.0.insert(id, PtyMaster::new(id, flags));

            Ok(id)
        } else {
            let master_id = path.parse::<usize>().or(Err(Error::new(EINVAL)))?;
            let master = self.ptys.0.get(&master_id).map(|pipe| pipe.clone()).ok_or(Error::new(ENOENT))?;

            let id = self.next_id;
            self.next_id += 1;

            self.ptys.1.insert(id, PtySlave::new(&master, flags));

            Ok(id)
        }
    }

    fn dup(&mut self, id: usize, _buf: &[u8]) -> Result<usize> {
        let master_opt = self.ptys.0.get(&id).map(|pipe| pipe.clone());
        if let Some(pipe) = master_opt {
            let pipe_id = self.next_id;
            self.next_id += 1;
            self.ptys.0.insert(pipe_id, pipe);
            return Ok(pipe_id);
        }

        let slave_opt = self.ptys.1.get(&id).map(|pipe| pipe.clone());
        if let Some(pipe) = slave_opt {
            let pipe_id = self.next_id;
            self.next_id += 1;
            self.ptys.1.insert(pipe_id, pipe);
            return Ok(pipe_id);
        }

        Err(Error::new(EBADF))
    }

    fn read(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let master_opt = self.ptys.0.get(&id).map(|pipe| pipe.clone());
        if let Some(pipe) = master_opt {
            return pipe.read(buf);
        }

        let slave_opt = self.ptys.1.get(&id).map(|pipe| pipe.clone());
        if let Some(pipe) = slave_opt {
            return pipe.read(buf);
        }

        Err(Error::new(EBADF))
    }

    fn write(&mut self, id: usize, buf: &[u8]) -> Result<usize> {
        let master_opt = self.ptys.0.get(&id).map(|pipe| pipe.clone());
        if let Some(pipe) = master_opt {
            return pipe.write(buf);
        }

        let slave_opt = self.ptys.1.get(&id).map(|pipe| pipe.clone());
        if let Some(pipe) = slave_opt {
            return pipe.write(buf);
        }

        Err(Error::new(EBADF))
    }

    fn fpath(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let master_opt = self.ptys.0.get(&id).map(|pipe| pipe.clone());
        if let Some(pipe) = master_opt {
            return pipe.path(buf);
        }

        let slave_opt = self.ptys.1.get(&id).map(|pipe| pipe.clone());
        if let Some(pipe) = slave_opt {
            return pipe.path(buf);
        }

        Err(Error::new(EBADF))
    }

    fn fsync(&mut self, id: usize) -> Result<usize> {
        let slave_opt = self.ptys.1.get(&id).map(|pipe| pipe.clone());
        if let Some(pipe) = slave_opt {
            return pipe.sync();
        }

        Ok(0)
    }

    fn close(&mut self, id: usize) -> Result<usize> {
        drop(self.ptys.0.remove(&id));
        drop(self.ptys.1.remove(&id));

        Ok(0)
    }
}

/// Read side of a pipe
#[derive(Clone)]
pub struct PtyMaster {
    id: usize,
    flags: usize,
    read: Arc<Mutex<VecDeque<Vec<u8>>>>,
    write: Arc<Mutex<VecDeque<u8>>>,
}

impl PtyMaster {
    pub fn new(id: usize, flags: usize) -> Self {
        PtyMaster {
            id: id,
            flags: flags,
            read: Arc::new(Mutex::new(VecDeque::new())),
            write: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let path_str = format!("pty:{}", self.id);
        let path = path_str.as_bytes();

        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }

        Ok(i)
    }

    fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let mut read = self.read.lock().unwrap();

        if let Some(packet) = read.pop_front() {
            let mut i = 0;

            while i < buf.len() && i < packet.len() {
                buf[i] = packet[i];
                i += 1;
            }

            Ok(i)
        } else if self.flags & O_NONBLOCK == O_NONBLOCK || Arc::weak_count(&self.read) == 0 {
            Ok(0)
        } else {
            Err(Error::new(EWOULDBLOCK))
        }
    }

    fn write(&self, buf: &[u8]) -> Result<usize> {
        let mut write = self.write.lock().unwrap();

        let mut i = 0;
        while i < buf.len() {
            write.push_back(buf[i]);
            i += 1;
        }

        Ok(i)
    }
}

/// Read side of a pipe
#[derive(Clone)]
pub struct PtySlave {
    master_id: usize,
    flags: usize,
    read: Weak<Mutex<VecDeque<u8>>>,
    write: Weak<Mutex<VecDeque<Vec<u8>>>>,
}

impl PtySlave {
    pub fn new(master: &PtyMaster, flags: usize) -> Self {
        PtySlave {
            master_id: master.id,
            flags: flags,
            read: Arc::downgrade(&master.write),
            write: Arc::downgrade(&master.read),
        }
    }

    fn path(&self, buf: &mut [u8]) -> Result<usize> {
        let path_str = format!("pty:{}", self.master_id);
        let path = path_str.as_bytes();

        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }

        Ok(i)
    }

    fn read(&self, buf: &mut [u8]) -> Result<usize> {
        if let Some(read_lock) = self.read.upgrade() {
            let mut read = read_lock.lock().unwrap();

            let mut i = 0;

            while i < buf.len() && ! read.is_empty() {
                buf[i] = read.pop_front().unwrap();
                i += 1;
            }

            if i > 0 || self.flags & O_NONBLOCK == O_NONBLOCK {
                Ok(i)
            } else {
                Err(Error::new(EWOULDBLOCK))
            }
        } else {
            Ok(0)
        }
    }

    fn write(&self, buf: &[u8]) -> Result<usize> {
        if let Some(write_lock) = self.write.upgrade() {
            let mut vec = Vec::new();
            vec.push(0);
            vec.extend_from_slice(buf);

            let mut write = write_lock.lock().unwrap();
            write.push_back(vec);

            Ok(buf.len())
        } else {
            Err(Error::new(EPIPE))
        }
    }

    fn sync(&self) -> Result<usize> {
        if let Some(write_lock) = self.write.upgrade() {
            let mut vec = Vec::new();
            vec.push(1);

            let mut write = write_lock.lock().unwrap();
            write.push_back(vec);

            Ok(0)
        } else {
            Err(Error::new(EPIPE))
        }
    }
}

fn main(){
    thread::spawn(move || {
        let mut socket = File::create(":pty").expect("pty: failed to create pty scheme");
        let mut scheme = PtyScheme::new();
        let mut todo = Vec::new();
        loop {
            let mut packet = Packet::default();
            socket.read(&mut packet).expect("pty: failed to read events from pty scheme");

            let a = packet.a;
            scheme.handle(&mut packet);
            if packet.a == (-EWOULDBLOCK) as usize {
                packet.a = a;
                todo.push(packet);
            } else {
                socket.write(&packet).expect("pty: failed to write responses to pty scheme");
            }

            let mut i = 0;
            while i < todo.len() {
                let a = todo[i].a;
                scheme.handle(&mut todo[i]);
                if todo[i].a == (-EWOULDBLOCK) as usize {
                    todo[i].a = a;
                    i += 1;
                } else {
                    let packet = todo.remove(i);
                    socket.write(&packet).expect("pty: failed to write responses to pty scheme");
                }
            }
        }
    });
}
