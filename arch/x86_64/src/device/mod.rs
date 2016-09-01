use paging::ActivePageTable;

pub mod display;
pub mod ps2;

pub unsafe fn init(active_table: &mut ActivePageTable){
    display::init(active_table);
    ps2::init();
}

pub unsafe fn init_ap(active_table: &mut ActivePageTable) {
    display::init_ap(active_table);
}
