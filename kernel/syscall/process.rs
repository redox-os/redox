///! Process syscalls

use core::str;

use arch;
use arch::paging::{VirtualAddress, entry};
use context;
use elf;
use syscall::{self, Error, Result};

pub fn brk(address: usize) -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::NoProcess)?;
    let mut context = context_lock.write();

    let mut current = arch::USER_HEAP_OFFSET;
    if let Some(ref heap) = context.heap {
        current = heap.start_address().get() + heap.size();
    }
    if address == 0 {
        //println!("Brk query {:X}", current);
        Ok(current)
    } else if address >= arch::USER_HEAP_OFFSET {
        //TODO: out of memory errors
        if let Some(ref mut heap) = context.heap {
            heap.resize(address - arch::USER_HEAP_OFFSET, true, true);
            return Ok(address);
        }

        context.heap = Some(context::memory::Memory::new(
            VirtualAddress::new(arch::USER_HEAP_OFFSET),
            address - arch::USER_HEAP_OFFSET,
            entry::WRITABLE | entry::NO_EXECUTE | entry::USER_ACCESSIBLE,
            true,
            true
        ));

        Ok(address)
    } else {
        //TODO: Return correct error
        Err(Error::NotPermitted)
    }
}

pub fn clone(flags: usize) -> Result<usize> {
    println!("Clone {:X}", flags);
    Ok(0)
}

pub fn exit(status: usize) -> ! {
    println!("Exit {}", status);

    {
        let contexts = context::contexts();
        let context_lock = contexts.current().expect("tried to exit without context");
        let mut context = context_lock.write();
        context.image.clear();
        drop(context.heap.take());
        drop(context.stack.take());
        context.exited = true;
    }

    unsafe { context::switch(); }

    unreachable!();
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
