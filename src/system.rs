/* ************************************************************************** */
/*                                                                            */
/*                          ____   __  __    ___  ______                      */
/*                         |  _ \ | | | |  / __| |_   _|                      */
/*                         | |/ / | |_| |  \__ \   | |                        */
/*                         |_|\_\  \___/  |___/    |_|                        */
/*                                                                            */
/*   File:     system.rs                Project:  proc-monitor                */
/*   Created:  2026-04-01               Updated:  2026-04-05                  */
/*   License:  MIT OR Apache-2.0                                              */
/*                                                                            */
/* ************************************************************************** */

use crate::process;
use process::{Process, get_pids};

use std::collections::HashMap;

pub fn refresh(
    map: &mut HashMap<u32, Process>,
    utime: f64,
    uids_table: &[(u32, String)],
) -> Result<(), ()> {
    let pids = get_pids();
    for pid in pids {
        if map.contains_key(&pid) {
            map.get_mut(&pid).unwrap().read_stat(utime)?;
        } else {
            if let Ok(process) = Process::new(pid, utime, &uids_table) {
                // println!("\n{process}");
                map.insert(pid, process);
            }
        }
    }
    Ok(())
}
