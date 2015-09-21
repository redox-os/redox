use common::context::*;
use common::elf::*;
use common::scheduler::*;

use programs::common::*;

pub struct Executor {
    executable: ELF
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            executable: ELF::new()
        }
    }
}

impl SessionItem for Executor {
    fn main(&mut self, url: URL){
        let mut resource = url.open();

        let mut vec: Vec<u8> = Vec::new();
        resource.read_to_end(&mut vec);

        unsafe{
            self.executable = ELF::from_data(vec.as_ptr() as usize);
            //self.executable.d();

            // Setup 4 MB upper mem space to map to program
            let reenable = start_no_ints();

            let contexts = &mut *(*contexts_ptr);

            match contexts.get(context_i) {
                Option::Some(mut current) => {
                    current.physical_address = self.executable.data + 4096; /*Extra 4096 for null segment*/
                    current.virtual_address = LOAD_ADDR;
                    current.virtual_size = 4 * 1024 * 1024; // 4 MB
                    current.map();
                },
                Option::None => ()
            }

            end_no_ints(reenable);

            let entry = self.executable.entry();
            if self.executable.can_call(entry){
                //Rediculous call mechanism
                let fn_ptr: *const usize = &entry;
                (*(fn_ptr as *const extern "cdecl" fn()))();
            }
        }
    }
}
