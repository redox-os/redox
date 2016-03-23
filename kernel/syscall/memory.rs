use arch::memory;

use system::error::Result;

//TODO: Refactor file to propogate results

pub fn do_sys_brk(addr: usize) -> Result<usize> {
    let mut ret = 0;

    let contexts = ::env().contexts.lock();
    if let Ok(current) = contexts.current() {
        ret = current.next_mem();

        // TODO: Make this smarter, currently it attempt to resize the entire data segment
        if let Some(mut mem) = unsafe { (*current.memory.get()).last_mut() } {
            if mem.writeable && mem.allocated {
                if addr >= mem.virtual_address {
                    unsafe { mem.unmap() };

                    let size = addr - mem.virtual_address;
                    let physical_address = unsafe { memory::realloc(mem.physical_address, size) };
                    if physical_address > 0 {
                        mem.physical_address = physical_address;
                        mem.virtual_size = size;
                        ret = mem.virtual_address + mem.virtual_size;
                    } else {
                        mem.virtual_size = 0;
                        debugln!("BRK: Realloc failed {:X}, {}\n", mem.virtual_address, size);
                    }

                    unsafe { mem.map() };
                }
            } else {
                debugln!("{:X}: {}", current.pid, current.name);
                debugln!("BRK: End segment not writeable or allocated");
            }
        } else {
            debugln!("BRK: No segments");
        }
    } else {
        debugln!("BRK: Context not found");
    }

    Ok(ret)
}
