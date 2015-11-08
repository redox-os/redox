use alloc::boxed::Box;

use collections::string::String;
use collections::vec::Vec;

use scheduler::context::{self, Context, ContextFile, ContextMemory};
use common::debug;
use common::elf::Elf;
use common::memory;
use common::parse_path::parse_path;
use common::pwd;
use scheduler;
use collections::string::ToString;

use schemes::Url;

/// Excecute an excecutable
//TODO: Modify current context, take current stdio
pub fn execute(url: &Url, wd: &Url, mut args: Vec<String>) -> Option<Box<Context>> {
    unsafe {
        let mut physical_address = 0;
        let mut virtual_address = 0;
        let mut virtual_size = 0;
        let mut entry = 0;

        if let Some(mut resource) = url.open() {
            let mut vec: Vec<u8> = Vec::new();
            resource.read_to_end(&mut vec);

            let executable = Elf::from_data(vec.as_ptr() as usize);
            if let Some(segment) = executable.load_segment() {
                virtual_address = segment.vaddr as usize;
                virtual_size = segment.mem_len as usize;
                physical_address = memory::alloc(virtual_size);

                if physical_address > 0 {
                    //Copy progbits
                    ::memcpy(physical_address as *mut u8, (executable.data + segment.off as usize) as *const u8, segment.file_len as usize);
                    //Zero bss
                    ::memset((physical_address + segment.file_len as usize) as *mut u8, 0, segment.mem_len as usize - segment.file_len as usize);
                }

                entry = executable.entry();
            } else {
                debug::d("Invalid ELF\n");
            }
        } else {
            for p in parse_path(url.reference(), pwd()) {
            debugln!("{}", p);
            }
            debug::d("Failed to open\n");
        }

        if physical_address > 0 && virtual_address > 0 && virtual_size > 0 &&
           entry >= virtual_address && entry < virtual_address + virtual_size {
            args.insert(0, url.to_string());

            let mut context_args: Vec<usize> = Vec::new();
            context_args.push(0); // ENVP
            context_args.push(0); // ARGV NULL
            let mut argc = 0;
            for i in 0..args.len() {
                if let Some(arg) = args.get(args.len() - i - 1) {
                    context_args.push(arg.as_ptr() as usize);
                    argc += 1;
                }
            }
            context_args.push(argc);

            let mut context = Context::new(entry, &context_args);

            //TODO: Push arg c_strs as things to clean up
            (*context.memory.get()).push(ContextMemory {
                physical_address: physical_address,
                virtual_address: virtual_address,
                virtual_size: virtual_size,
            });

            *context.cwd.get() = wd.to_string();

            *context.args.get() = args;

            if let Some(stdin) = Url::from_str("debug://").open() {
                (*context.files.get()).push(ContextFile {
                    fd: 0, // STDIN
                    resource: stdin,
                });
            } else {
                debugln!("Failed to open stdin");
            }

            if let Some(stdout) = Url::from_str("debug://").open() {
                (*context.files.get()).push(ContextFile {
                    fd: 1, // STDOUT
                    resource: stdout,
                });
            } else {
                debugln!("Failed to open stdout");
            }

            if let Some(stderr) = Url::from_str("debug://").open() {
                (*context.files.get()).push(ContextFile {
                    fd: 2, // STDERR
                    resource: stderr,
                });
            } else {
                debugln!("Failed to open stderr");
            }

            return Some(context);
        } else {
            debug::d("Invalid entry\n");

            if physical_address > 0 {
                memory::unalloc(physical_address);
            }
        }
    }

    None
}
