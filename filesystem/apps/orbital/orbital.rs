use redox::*;
pub fn main() {
    /*
    match Session::new() {
        Some(_) => {
            if let Some(mut display) = File::open("display://") {
                let colors:Vec<u32> = vec![0xFF00FF00; 640*480];
                unsafe { 
                    let u8s = mem::transmute::<&[u32],&[u8]>(&colors[..]); 
                    display.write(u8s);
                    display.sync();
                }
            }
        },
        None => {
            if let Some(mut display) = File::open("display://") {
                let colors:Vec<u32> = vec![0xFF0000FF; 640*480];
                unsafe { 
                    let u8s = mem::transmute::<&[u32],&[u8]>(&colors[..]); 
                    display.write(u8s);
                    display.sync();
                }
            }
        },
    }
    */
    unsafe {
        Session::exec();
    }
    loop {}
    /*
    // THIS WORKS
    if let Some(mut display) = File::open("display://") {
        let colors:Vec<u32> = vec![0xFF00FF00; 640*480];
        unsafe { 
            let u8s = mem::transmute::<&[u32],&[u8]>(&colors[..]); 
            display.write(u8s);
            display.sync();
        }
    }
    loop {}
    */


    /*
    // THIS KINDA WORKS, the colors are off it's rgba instead of abgr
    if let Some(mut display) = File::open("display://") {
            let stop = 640*300;
            for i in 0..stop {
                display.write(&[0xFF,0x00,0xFF,0x00]);
            }
            display.sync();
    }
    loop {}
    */
}
