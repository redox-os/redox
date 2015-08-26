pub fn sched_yield(){
    unsafe {
        asm!("int 0x80"
            : : "{eax}"(3) : : "intel");
    }
}
