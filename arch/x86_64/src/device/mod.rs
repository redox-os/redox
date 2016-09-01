use paging::ActivePageTable;

pub mod display;
pub mod ps2;
pub mod serial;

pub unsafe fn init(active_table: &mut ActivePageTable){
    serial::init();
    ps2::init();
    display::init(active_table);
}

pub unsafe fn init_ap(active_table: &mut ActivePageTable) {
    display::init_ap(active_table);
}
