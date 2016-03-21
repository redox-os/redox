use arch::context::ContextMemory;
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

pub fn do_sys_alloc(size: usize) -> Result<usize> {
    let mut ret = 0;

    let contexts = ::env().contexts.lock();
    if let Ok(current) = contexts.current() {
        let physical_address = unsafe { memory::alloc(size) };
        if physical_address > 0 {
            ret = current.next_mem();

            let mut mem = ContextMemory {
                physical_address: physical_address,
                virtual_address: ret,
                virtual_size: size,
                writeable: true,
                allocated: true,
            };

            //debugln!("{}: {}: allocate {:X}:{:X}", current.pid, current.name, mem.virtual_address, mem.virtual_address + mem.virtual_size);

            unsafe {
                mem.map();
                (*current.memory.get()).push(mem);
            }
        }
    }

    Ok(ret)
}

pub fn do_sys_realloc(ptr: usize, size: usize) -> Result<usize> {
    let mut ret = 0;

    let mut contexts = ::env().contexts.lock();
    if let Ok(mut current) = contexts.current_mut() {
        if let Ok(mut mem) = current.get_mem_mut(ptr) {
            unsafe { mem.unmap(); }

            //debug!("{}: {}: reallocate {:X}:{:X}", current.pid, current.name, mem.virtual_address, mem.virtual_address + mem.virtual_size);

            let physical_address = unsafe { memory::realloc(mem.physical_address, size) };
            if physical_address > 0 {
                mem.physical_address = physical_address;
                mem.virtual_size = size;
                ret = mem.virtual_address;
            } else {
                mem.virtual_size = 0;
            }

            //debugln!(" to {:X}:{:X}", mem.virtual_address, mem.virtual_address + mem.virtual_size);

            unsafe { mem.map(); }
        }
        unsafe { current.clean_mem(); }
    }

    Ok(ret)
}

pub fn do_sys_realloc_inplace(ptr: usize, size: usize) -> Result<usize> {
    let mut ret = 0;

    let mut contexts = ::env().contexts.lock();
    if let Ok(mut current) = contexts.current_mut() {
        if let Ok(mut mem) = current.get_mem_mut(ptr) {
            unsafe { mem.unmap(); }

            //debug!("{}: {}: reallocate {:X}:{:X}", current.pid, current.name, mem.virtual_address, mem.virtual_address + mem.virtual_size);

            mem.virtual_size = unsafe { memory::realloc_inplace(mem.physical_address, size) };
            ret = mem.virtual_size;

            //debugln!(" to {:X}:{:X}", mem.virtual_address, mem.virtual_address + mem.virtual_size);

            unsafe {mem.map(); }
        }
        unsafe { current.clean_mem(); }
    }

    Ok(ret)
}

pub fn do_sys_unalloc(ptr: usize) -> Result<usize> {
    let mut contexts = ::env().contexts.lock();
    if let Ok(mut current) = contexts.current_mut() {
        if let Ok(mut mem) = current.get_mem_mut(ptr) {
            unsafe { mem.unmap() };

            //debugln!("{}: {}: unallocate {:X}:{:X}", current.pid, current.name, mem.virtual_address, mem.virtual_address + mem.virtual_size);

            mem.virtual_size = 0;
        }
        unsafe { current.clean_mem(); }
    }
    Ok(0)
}
