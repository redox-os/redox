//! System calls for basic memory management.

use arch::context::ContextMemory;
use arch::memory;

use system::error::Result;

//TODO: Refactor file to propogate results

pub fn brk(addr: usize) -> Result<usize> {
    let mut ret = 0;

    let contexts = unsafe { & *::env().contexts.get() };
    if let Ok(current) = contexts.current() {
        ret = unsafe { (*current.heap.get()).address };

        for mem in unsafe { (*current.heap.get()).memory.iter() } {
            let pages = (mem.virtual_size + 4095) / 4096;
            let end = mem.virtual_address + pages * 4096;
            if end > ret {
                ret = end;
            }
        }

        if addr == 0 {
            //Return current break
        } else if addr > ret {
            let size = addr - ret;
            let physical_address = unsafe { memory::alloc_aligned(size, 4096) };
            if physical_address > 0 {
                // debugln!("BRK: Alloc {}", size);
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
                debugln!("BRK: Alloc failed {}", size);
            }
        } else if addr < ret {
            //TODO: Realloc
            let mut clean = false;
            for mut mem in unsafe { (*current.heap.get()).memory.iter_mut() } {
                if addr <= mem.virtual_address {
                    unsafe { mem.unmap() };
                    mem.virtual_size = 0;
                    clean = true;
                }
            }
            if clean {
                unsafe { (*current.heap.get()).clean_mem() };
            }
        } else {
            //Already set to desired break
        }
    } else {
        debugln!("BRK: Context not found");
    }

    Ok(ret)
}
