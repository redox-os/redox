use super::*;
use collections::string::String;
use syscall::handle::*;
use syscall::common::*;

/// Run command in system console
pub fn run(c: String) {
    let mut atms = c.split(' ');
    
    unsafe {

        match atms.nth(0) {
            Some("test") => {
                debugln!("All clear!");
            },
            Some("help") => {
                debugln!("This is the Redox system console.");
                debugln!("Following commands are valid:");
                debugln!("- test");
                debugln!("- help");
                debugln!("- cd");
                debugln!("- time");
                debugln!("- halt");
            },
            Some("cd") => {
                do_sys_chdir(atms.nth(0).unwrap_or("/").as_ptr());
            },
            Some("time") => {
                debugln!("{}", do_sys_clock_gettime(0, Regs::default().cx as *mut TimeSpec));
            },
            Some("halt") => {
                debugln!("Halting...");
                do_sys_exit(0);
            },
            //        Some("sleep") => {
//            do_sys_nanosleep(c.nth(0).to_num());
//        }
        None => {
            debugln!("No command given...");
        },
        _ => {
            debugln!("Unknown command. Type 'help' for help.");
        },

        }
    }

}
