use programs::common::*;

pub struct DebugResource;

impl Resource for DebugResource {
    fn url(&self) -> URL {
        return URL::from_str("debug://");
    }

    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        return Option::None;
    }

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        return Option::None;
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        for byte in buf {
            unsafe{
                sys_debug(*byte);
            }
        }
        return Option::Some(buf.len());
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None;
    }

    fn flush(&mut self) -> bool {
        return true;
    }
}

pub struct DebugScheme;

impl SessionItem for DebugScheme {
    fn scheme(&self) -> String {
        return "debug".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource>{
        return box DebugResource;
    }
}
