use programs::common::*;

struct DebugResource;

impl Resource for DebugResource {
    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        for b in buf {
            db(*b);
        }
        return Option::Some(buf.len());
    }
}

pub struct DebugScheme;

impl SessionItem for DebugScheme {
    fn scheme(&self) -> String {
        return "debug".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        return box DebugResource;
    }
}
