#[derive(Debug, Default, RustcDecodable)]
pub struct Config {
    pub drivers: Vec<DriverConfig>
}

#[derive(Debug, Default, RustcDecodable)]
pub struct DriverConfig {
    pub name: Option<String>,
    pub class: Option<u8>,
    pub subclass: Option<u8>,
    pub vendor: Option<u16>,
    pub device: Option<u16>,
    pub command: Option<Vec<String>>
}
