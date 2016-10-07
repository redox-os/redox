pub mod rtc;
pub mod serial;

pub unsafe fn init(){
    rtc::init();
    serial::init();
}
