pub unsafe fn inb(port: u16) -> u8{
	let ret: u8;
    asm!("in $0, $1" : "={al}"(ret) : "{dx}"(port) : : "intel", "volatile");
	ret
}

pub unsafe fn inw(port: u16) -> u16{
    let ret: u16;
    asm!("in $0, $1" : "={ax}"(ret) : "{dx}"(port) : : "intel", "volatile");
    ret
}

pub unsafe fn ind(port: u16) -> u32{
    let ret: u32;
    asm!("in $0, $1" : "={eax}"(ret) : "{dx}"(port) : : "intel", "volatile");
    ret
}

pub unsafe fn outb(port: u16, value: u8){
    asm!("out $1, $0" : : "{al}"(value), "{dx}"(port) : : "intel", "volatile");
}

pub unsafe fn outw(port: u16, value: u16){
    asm!("out $1, $0" : : "{ax}"(value), "{dx}"(port) : : "intel", "volatile");
}

pub unsafe fn outd(port: u16, value: u32){
    asm!("out $1, $0" : : "{eax}"(value), "{dx}"(port) : : "intel", "volatile");
}
