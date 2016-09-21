use paging::ActivePageTable;

pub mod display;
pub mod serial;

pub unsafe fn init(active_table: &mut ActivePageTable){
    serial::init();
    display::init(active_table);
}
