//A test of ANSI capabilities

fn sleep(){
    ::std::thread::sleep_ms(1000);
}

fn main(){
    println!("Removed by reset");
    sleep();
    println!("\x1BcReset. Removed by cursor home");
    sleep();
    println!("\x1B[HCursor Home. At start of screen");
    sleep();
    println!("\x1B[12;40HCursor Home. At middle of screen");
    sleep();
    println!("\x1B[24;0fAlternate cursor home. At end of screen");
}
