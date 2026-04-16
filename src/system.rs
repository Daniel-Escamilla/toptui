/* ************************************************************************** */
/*                                                                            */
/*                          ____   __  __    ___  ______                      */
/*                         |  _ \ | | | |  / __| |_   _|                      */
/*                         | |/ / | |_| |  \__ \   | |                        */
/*                         |_|\_\  \___/  |___/    |_|                        */
/*                                                                            */
/*   File:     system.rs                Project:  toptui                      */
/*   Created:  2026-04-02               Updated:  2026-04-16                  */
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
    ticks: f64,
    refresh_seconds: f64,
) -> Result<(), ()> {
    let pids = get_pids();
    for pid in pids {
        if let std::collections::hash_map::Entry::Vacant(e) = map.entry(pid) {
            if let Ok(process) = Process::new(pid, utime, uids_table, ticks, refresh_seconds) {
                // println!("\n{process}");
                e.insert(process);
            }
        } else {
            map.get_mut(&pid)
                .unwrap()
                .read_stat(utime, ticks, refresh_seconds)?;
        }
    }
    Ok(())
}
