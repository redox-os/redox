use common::context::*;
use common::elf::*;
use common::scheduler::*;

use programs::common::*;

pub struct Executor {
    executable: ELF,
    entry: usize,
    exit: usize
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            executable: ELF::new(),
            entry: 0,
            exit: 0
        }
    }
}

impl Drop for Executor {
    fn drop(&mut self){
        unsafe{
            if self.executable.can_call(self.exit){
                //Rediculous call mechanism
                let fn_ptr: *const usize = &self.exit;
                (*(fn_ptr as *const fn()))();
            }
        }
    }
}

impl SessionItem for Executor {
    fn main(&mut self, url: URL){
        let mut resource = url.open();

        let mut vec: Vec<u8> = Vec::new();
        match resource.read_to_end(&mut vec){
            Option::Some(_) => {
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

                    self.entry = self.executable.entry();
                    self.exit = self.executable.symbol("exit".to_string());

                    if self.executable.can_call(self.entry){
                        //Rediculous call mechanism
                        let fn_ptr: *const usize = &self.entry;
                        (*(fn_ptr as *const fn()))();
                    }
                }
            },
            Option::None => ()
        }
    }
}
