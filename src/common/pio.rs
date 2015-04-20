pub unsafe fn inb(port: u16) -> u8{
	let ret: u8;
    asm!("in $0, $1\n"
        : "={al}"(ret) : "{dx}"(port) : : "intel");
	ret
}

pub unsafe fn inw(port: u16) -> u16{
    let ret: u16;
    asm!("in $0, $1\n"
        : "={ax}"(ret) : "{dx}"(port) : : "intel");
    ret
}

pub unsafe fn outb(port: u16, value: u8){
    asm!("out $1, $0\n"
        : : "{al}"(value), "{dx}"(port) : : "intel");
}

pub unsafe fn outw(port: u16, value: u16){
    asm!("out $1, $0\n"
        : : "{ax}"(value), "{dx}"(port) : : "intel");
}