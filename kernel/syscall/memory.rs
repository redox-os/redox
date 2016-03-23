use arch::memory;

use system::error::Result;

//TODO: Refactor file to propogate results

pub fn do_sys_brk(addr: usize) -> Result<usize> {
    let mut ret = 0;

    let mut contexts = ::env().contexts.lock();
    if let Ok(mut current) = contexts.current_mut() {
        unsafe {
            current.unmap();
        }

        ret = current.next_mem();

        // TODO: Make this smarter, currently it attempt to resize the entire data segment
        if let Some(mut mem) = unsafe { (*current.memory.get()).last_mut() } {
            if mem.writeable && mem.allocated {
                if addr >= mem.virtual_address {
                    let size = addr - mem.virtual_address;
                    let physical_address = unsafe { memory::realloc(mem.physical_address, size) };
                    if physical_address > 0 {
                        mem.physical_address = physical_address;
                        mem.virtual_size = size;
                        ret = mem.virtual_address + mem.virtual_size;
                    } else {
                        mem.virtual_size = 0;
                        debug!("BRK: Realloc failed {:X}, {}\n", mem.virtual_address, size);
                    }
                }
            } else {
                debug!("BRK: End segment not writeable or allocated\n");
            }
        } else {
            debug!("BRK: No segments\n")
        }

        unsafe {
            current.clean_mem();
            current.map();
        }
    } else {
        debug!("BRK: Context not found\n");
    }

    Ok(ret)
}
