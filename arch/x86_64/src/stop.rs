use io::{Io, Pio};

#[no_mangle]
pub unsafe extern fn kstop() -> ! {
    // (phony) ACPI shutdown (http://forum.osdev.org/viewtopic.php?t=16990)
    // Works for qemu and bochs.
    for &port in [0x604, 0xB004].iter() {
        println!("Shutdown with outw(0x{:X}, 0x{:X})", port, 0x2000);
        Pio::<u16>::new(port).write(0x2000);
    }

    // Magic shutdown code for bochs and qemu (older versions).
    for c in "Shutdown".bytes() {
        println!("Shutdown with outb(0x{:X}, '{}')", 0x8900, c as char);
        Pio::<u8>::new(0x8900).write(c);
    }

    // Magic code for VMWare. Also a hard lock.
    println!("Shutdown with cli hlt");
    asm!("cli; hlt" : : : : "intel", "volatile");

    unreachable!();
}
