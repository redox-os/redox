use syscall::call::*;

struct File {
    path: String,
    fd: usize
}

impl File {
    pub fn open(path: &str) -> Box<File> {
        let c_str: *const u8 = path.to_c_str();
        let ret = box File {
            path: path.clone(),
            fd: sys_open(c_str, 0, 0)
        };
        unalloc(c_str as usize);

        return ret;
    }
}

impl Resource for File {
    fn url(&self) -> URL {
        //TODO
        return self.path.clone();
    }

    fn stat(&self) -> ResourceType {
        //TODO
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        unsafe{
            let count = sys_read(self.fd, buf.as_mut_ptr(), buf.len());
            if count == 0xFFFFFFFF {
                return Option::None;
            }else{
                return Option::Some(count);
            }
        }
    }

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        unsafe{
            //TODO: Replace
            let count = sys_read_to_end(self.fd, vec as *mut Vec<u8>);
            if count == 0xFFFFFFFF {
                return Option::None;
            }else{
                return Option::Some(count);
            }
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        unsafe{
            let count = sys_write(self.fd, buf.as_ptr(), buf.len());
            if count == 0xFFFFFFFF {
                return Option::None;
            }else{
                return Option::Some(count);
            }
        }
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None;
    }

    fn flush(&mut self) -> bool {
        return false;
    }
}

impl Drop for File {
    fn drop(&mut self){
        unsafe{
            sys_close(self.fd);
        }
    }
}
