use collections::{String, Vec};
use core::str;

use context;
use syscall::error::Result;

pub fn resource() -> Result<Vec<u8>> {
    let mut string = format!("{:<6}{:<6}{:<6}{:<6}{:<6}{:<6}{:<6}{:<6}{:<6}{:<6}{:<8}{}\n",
                             "PID",
                             "PPID",
                             "RUID",
                             "RGID",
                             "RNS",
                             "EUID",
                             "EGID",
                             "ENS",
                             "STAT",
                             "CPU",
                             "MEM",
                             "NAME");
    {
        let contexts = context::contexts();
        for (_id, context_lock) in contexts.iter() {
            let context = context_lock.read();

            let mut stat_string = String::new();
            if context.stack.is_some() {
                stat_string.push('U');
            } else {
                stat_string.push('K');
            }
            match context.status {
                context::Status::Runnable => {
                    stat_string.push('R');
                },
                context::Status::Blocked => if context.wake.is_some() {
                    stat_string.push('S');
                } else {
                    stat_string.push('B');
                },
                context::Status::Exited(_status) => {
                    stat_string.push('Z');
                }
            }
            if context.running {
                stat_string.push('+');
            }

            let cpu_string = if let Some(cpu_id) = context.cpu_id {
                format!("{}", cpu_id)
            } else {
                format!("?")
            };

            let mut memory = 0;
            if let Some(ref kfx) = context.kstack {
                memory += kfx.len();
            }
            if let Some(ref kstack) = context.kstack {
                memory += kstack.len();
            }
            for shared_mem in context.image.iter() {
                shared_mem.with(|mem| {
                    memory += mem.size();
                });
            }
            if let Some(ref heap) = context.heap {
                heap.with(|heap| {
                    memory += heap.size();
                });
            }
            if let Some(ref stack) = context.stack {
                memory += stack.size();
            }

            let memory_string = if memory >= 1024 * 1024 * 1024 {
                format!("{} GB", memory / 1024 / 1024 / 1024)
            } else if memory >= 1024 * 1024 {
                format!("{} MB", memory / 1024 / 1024)
            } else if memory >= 1024 {
                format!("{} KB", memory / 1024)
            } else {
                format!("{} B", memory)
            };

            let name_bytes = context.name.lock();
            let name = str::from_utf8(&name_bytes).unwrap_or("");

            string.push_str(&format!("{:<6}{:<6}{:<6}{:<6}{:<6}{:<6}{:<6}{:<6}{:<6}{:<6}{:<8}{}\n",
                               context.id.into(),
                               context.ppid.into(),
                               context.ruid,
                               context.rgid,
                               context.rns.into(),
                               context.euid,
                               context.egid,
                               context.ens.into(),
                               stat_string,
                               cpu_string,
                               memory_string,
                               name));
        }
    }

    Ok(string.into_bytes())
}
