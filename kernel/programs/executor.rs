use collections::string::String;
use collections::vec::Vec;

use scheduler::context::{contexts_ptr, Context, ContextFile, ContextMemory};
use common::debug;
use common::elf::Elf;
use common::memory;
use scheduler;
use collections::string::ToString;

use schemes::Url;

/// Excecute an excecutable
// TODO: Modify current context, take current stdio
pub fn execute(url: &Url, wd: &Url, mut args: Vec<String>) {
    unsafe {
        let mut memory = Vec::new();
        let mut entry = 0;

        if let Some(mut resource) = url.open() {
            let mut vec: Vec<u8> = Vec::new();
            resource.read_to_end(&mut vec);

            let executable = Elf::from_data(vec.as_ptr() as usize);
            entry = executable.entry();
            for segment in executable.load_segment().iter() {
                let virtual_address = segment.vaddr as usize;
                let virtual_size = segment.mem_len as usize;
                let physical_address = memory::alloc(virtual_size);

                if physical_address > 0 {
                    // Copy progbits
                    ::memcpy(physical_address as *mut u8,
                             (executable.data + segment.off as usize) as *const u8,
                             segment.file_len as usize);
                    // Zero bss
                    ::memset((physical_address + segment.file_len as usize) as *mut u8,
                             0,
                             segment.mem_len as usize - segment.file_len as usize);

                    memory.push(ContextMemory {
                        physical_address: physical_address,
                        virtual_address: virtual_address,
                        virtual_size: virtual_size,
                        writeable: segment.flags & 2 == 2,
                    });
                }
            }
        } else {
            debug::d("Failed to open\n");
        }

        if !memory.is_empty() && entry > 0 {
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

            let context = Context::new(url.to_string(), true, entry, &context_args);

            (*context.memory.get()) = memory;

            *context.cwd.get() = wd.to_string();

            *context.args.get() = args;

            // TODO: Do this the right way
            let mut create = [true; 3];
            if let Some(current) = Context::current() {
                if let Some(stdin) = current.get_file(0) {
                    if let Some(stdin_dup) = stdin.dup() {
                        (*context.files.get()).push(ContextFile {
                            fd: 0, // STDIN
                            resource: stdin_dup,
                        });
                        create[0] = false;
                    }
                }

                if let Some(stdout) = current.get_file(1) {
                    if let Some(stdout_dup) = stdout.dup() {
                        (*context.files.get()).push(ContextFile {
                            fd: 1, // STDOUT
                            resource: stdout_dup,
                        });
                        create[1] = false;
                    }
                }

                if let Some(stderr) = current.get_file(2) {
                    if let Some(stderr_dup) = stderr.dup() {
                        (*context.files.get()).push(ContextFile {
                            fd: 2, // STDERR
                            resource: stderr_dup,
                        });
                        create[2] = false;
                    }
                }
            }

            if create[0] {
                if let Some(stdin) = Url::from_str("debug:").open() {
                    (*context.files.get()).push(ContextFile {
                        fd: 0, // STDIN
                        resource: stdin,
                    });
                } else {
                    debugln!("Failed to open stdin");
                }
            }

            if create[1] {
                if let Some(stdout) = Url::from_str("debug:").open() {
                    (*context.files.get()).push(ContextFile {
                        fd: 1, // STDOUT
                        resource: stdout,
                    });
                } else {
                    debugln!("Failed to open stdout");
                }
            }

            if create[2] {
                if let Some(stderr) = Url::from_str("debug:").open() {
                    (*context.files.get()).push(ContextFile {
                        fd: 2, // STDERR
                        resource: stderr,
                    });
                } else {
                    debugln!("Failed to open stderr");
                }
            }

            let reenable = scheduler::start_no_ints();
            (*contexts_ptr).push(context);
            scheduler::end_no_ints(reenable);
        } else {
            debug::d("Invalid memory or entry\n");
        }
    }
}
