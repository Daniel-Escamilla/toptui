/* ************************************************************************** */
/*                                                                            */
/*                          ____   __  __    ___  ______                      */
/*                         |  _ \ | | | |  / __| |_   _|                      */
/*                         | |/ / | |_| |  \__ \   | |                        */
/*                         |_|\_\  \___/  |___/    |_|                        */
/*                                                                            */
/*   File:     system.rs                Project:  toptui                      */
/*   Created:  2026-04-02               Updated:  2026-04-19                  */
/*   License:  MIT OR Apache-2.0                                              */
/*                                                                            */
/* ************************************************************************** */

use crate::process;
use process::{Process, get_pids};

use crate::fs::{self};

use std::collections::HashMap;

#[derive(Default)]
#[derive(Debug)]
struct MemoryData {
    mem_total: u64,
    mem_available: u64,
    swap_total: u64,
    swap_free: u64,
}

fn read_meminfo() -> Result<MemoryData, ()> {
    let mut data: MemoryData = Default::default();
    let meminfo_file = fs::read_to_string("/proc/meminfo").map_err(|_| ())?;
    for line in meminfo_file.lines() {
        let Some(category) = line.split(':').nth(0) else {
            continue;
        };
        let category = category.trim();
        let Some(value) = line.split_whitespace().nth(1) else {
            continue;
        };
        match category {
            "MemTotal" => data.mem_total = value.parse::<u64>().map_err(|_| ())?,
            "MemAvailable" => data.mem_available = value.parse::<u64>().map_err(|_| ())?,
            "SwapTotal" => data.swap_total = value.parse::<u64>().map_err(|_| ())?,
            "SwapFree" => data.swap_free = value.parse::<u64>().map_err(|_| ())?,
            _ => {}
        }
    }
    Ok(data)
}

pub fn refresh(
    map: &mut HashMap<u32, Process>,
    utime: f64,
    uids_table: &[(u32, String)],
    ticks: f64,
    refresh_seconds: f64,
) -> Result<(), ()> {
    let pids = get_pids();
    let mem = read_meminfo()?;
    for pid in pids {
        if let std::collections::hash_map::Entry::Vacant(e) = map.entry(pid) {
            if let Ok(process) = Process::new(pid, utime, uids_table, ticks, refresh_seconds) {
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
