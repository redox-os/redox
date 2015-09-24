use common::context::*;
use common::elf::*;
use common::memory::*;
use common::scheduler::*;

use programs::common::*;

pub struct Executor;

impl Executor {
    pub fn new() -> Executor {
        Executor
    }
}

impl SessionItem for Executor {
    fn main(&mut self, url: URL){
        unsafe{
            let mut physical_address = 0;
            let virtual_address = LOAD_ADDR;
            let mut virtual_size = 0;

            let mut entry = 0;
            {
                let mut resource = url.open();
                drop(url);

                let mut vec: Vec<u8> = Vec::new();
                resource.read_to_end(&mut vec);
                drop(resource);

                let executable = ELF::from_data(vec.as_ptr() as usize);
                drop(vec);

                if executable.data > 0 {
                    virtual_size = alloc_size(executable.data) - 4096;
                    physical_address = alloc(virtual_size);
                    ptr::copy((executable.data + 4096) as *const u8, physical_address as *mut u8, virtual_size);
                    entry = executable.entry();
                }
                drop(executable);
            }

            if physical_address > 0 && virtual_address > 0 && virtual_size > 0 && entry >= virtual_address && entry < virtual_address + virtual_size {
                let reenable = start_no_ints();

                let contexts = &mut *(*contexts_ptr);

                match contexts.get(context_i) {
                    Option::Some(mut current) => {
                        current.memory.push(ContextMemory {
                            physical_address: physical_address,
                            virtual_address: virtual_address,
                            virtual_size: virtual_size
                        });
                        current.map();
                    },
                    Option::None => ()
                }

                end_no_ints(reenable);

                let fn_ptr: *const usize = &entry;
                (*(fn_ptr as *const extern "cdecl" fn()))();
            }else if physical_address > 0{
                unalloc(physical_address);
            }
        }
    }
}
