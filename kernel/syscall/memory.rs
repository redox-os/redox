use arch::context::ContextMemory;
use arch::memory;

use system::error::Result;

//TODO: Refactor file to propogate results

pub fn do_sys_brk(addr: usize) -> Result<usize> {
    let mut ret = 0;

    let contexts = ::env().contexts.lock();
    if let Ok(current) = contexts.current() {
        ret = unsafe { (*current.heap.get()).next_mem() };

        // TODO: Make this smarter, currently it attempt to resize the entire data segment
        if let Some(mut mem) = unsafe { (*current.heap.get()).memory.last_mut() } {
            if mem.writeable && mem.allocated {
                if addr >= mem.virtual_address {
                    unsafe { mem.unmap() };

                    let size = addr - mem.virtual_address;
                    let physical_address = unsafe { memory::realloc_aligned(mem.physical_address, size, 4096) };
                    if physical_address > 0 {
                        mem.physical_address = physical_address;
                        mem.virtual_size = size;
                        ret = mem.virtual_address + mem.virtual_size;
                    } else {
                        debugln!("BRK: Realloc failed {:X}, {}\n", mem.virtual_address, size);
                    }

                    unsafe { mem.map() };
                }
            } else {
                debugln!("{}: {}", current.pid, current.name);
                debugln!("BRK: End segment not writeable or allocated");
            }
        } else if addr >= ret {
            let size = addr - ret;
            let physical_address = unsafe { memory::alloc_aligned(size, 4096) };
            if physical_address > 0 {
                let mut mem = ContextMemory {
                    physical_address: physical_address,
                    virtual_address: ret,
                    virtual_size: size,
                    writeable: true,
                    allocated: true
                };
                ret = mem.virtual_address + mem.virtual_size;

                unsafe {
                    mem.map();
                    (*current.heap.get()).memory.push(mem);
                }
            } else {
                debugln!("BRK: Alloc failed {}\n", size);
            }
        }
    } else {
        debugln!("BRK: Context not found");
    }

    Ok(ret)
}
