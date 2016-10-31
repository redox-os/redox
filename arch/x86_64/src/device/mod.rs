use paging::ActivePageTable;

pub mod local_apic;
pub mod rtc;
pub mod serial;

pub unsafe fn init(active_table: &mut ActivePageTable){
    local_apic::init(active_table);
    rtc::init();
    serial::init();
}

pub unsafe fn init_ap() {
    local_apic::init_ap();
}
