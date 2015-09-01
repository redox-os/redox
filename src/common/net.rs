use core::result::Result;

use common::string::*;

pub struct TcpStream;

pub struct TcpListener {
    port: u16
}

impl TcpListener {
    pub fn bind(port: usize) -> Result<TcpListener, String> {
        return Result::Err("Bind not implemented".to_string());
    }

    pub fn accept(&self) -> Result<TcpStream, String> {
        return Result::Err("Accept not implemented".to_string());
    }
}
