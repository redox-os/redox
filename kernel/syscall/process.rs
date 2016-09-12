///! Process syscalls

use core::str;

use arch;
use arch::interrupt::halt;
use arch::paging::{ActivePageTable, Page, VirtualAddress, entry};
use context;
use elf;
use syscall::{self, Error, Result};

pub fn brk(address: usize) -> Result<usize> {
    //TODO: Make this more efficient
    let mut active_table = unsafe { ActivePageTable::new() };
    let mut current = arch::USER_HEAP_OFFSET;
    {
        let min_page = Page::containing_address(VirtualAddress::new(arch::USER_HEAP_OFFSET));
        let max_page = Page::containing_address(VirtualAddress::new(arch::USER_HEAP_OFFSET + arch::USER_HEAP_SIZE - 1));
        for page in Page::range_inclusive(min_page, max_page) {
            if active_table.translate_page(page).is_none() {
                break;
            }
            current = page.start_address().get() + 4096;
        }
    }
    if address == 0 {
        //println!("Brk query {:X}", current);
        Ok(current)
    } else if address > current {
        let start_page = Page::containing_address(VirtualAddress::new(current));
        let end_page = Page::containing_address(VirtualAddress::new(address - 1));
        for page in Page::range_inclusive(start_page, end_page) {
            //println!("Map {:X}", page.start_address().get());
            if active_table.translate_page(page).is_none() {
                //println!("Not found - mapping");
                active_table.map(page, entry::PRESENT | entry::WRITABLE | entry::NO_EXECUTE | entry::USER_ACCESSIBLE);
                active_table.flush(page);
            } else {
                //println!("Found - skipping");
            }
        }
        //let new = end_page.start_address().get() + 4096;
        //println!("Brk increase {:X}: from {:X} to {:X}", address, current, new);
        Ok(address)
    } else {
        let start_page = Page::containing_address(VirtualAddress::new(address));
        let end_page = Page::containing_address(VirtualAddress::new(current - 1));
        for page in Page::range_inclusive(start_page, end_page) {
            //println!("Unmap {:X}", page.start_address().get());
            if active_table.translate_page(page).is_some() {
                //println!("Found - unmapping");
                active_table.unmap(page);
                active_table.flush(page);
            } else {
                //println!("Not found - skipping");
            }
        }
        //let new = start_page.start_address().get();
        //println!("Brk decrease {:X}: from {:X} to {:X}", address, current, new);
        Ok(address)
    }
}

pub fn clone(flags: usize) -> Result<usize> {
    println!("Clone {:X}", flags);
    Ok(0)
}

pub fn exit(status: usize) -> ! {
    println!("Exit {}", status);
    loop {
        unsafe { halt() };
    }
}

pub fn exec(path: &[u8], _args: &[[usize; 2]]) -> Result<usize> {
    //TODO: Use args
    //TODO: Unmap previous mappings
    //TODO: Drop data vec
    println!("Exec {}", unsafe { str::from_utf8_unchecked(path) });

    let file = syscall::open(path, 0)?;
    let mut data = vec![];
    loop {
        let mut buf = [0; 4096];
        let count = syscall::read(file, &mut buf)?;
        if count > 0 {
            data.extend_from_slice(&buf[..count]);
        } else {
            break;
        }
    }
    let _ = syscall::close(file);

    match elf::Elf::from(&data) {
        Ok(elf) => elf.run().and(Ok(0)),
        Err(err) => {
            println!("failed to execute {}: {}", unsafe { str::from_utf8_unchecked(path) }, err);
            Err(Error::NoExec)
        }
    }
}

pub fn getpid() -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::NoProcess)?;
    let context = context_lock.read();
    Ok(context.id)
}

pub fn iopl(_level: usize) -> Result<usize> {
    //TODO
    Ok(0)
}

pub fn sched_yield() -> Result<usize> {
    unsafe { context::switch(); }
    Ok(0)
}
