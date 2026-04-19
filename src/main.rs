/* ************************************************************************** */
/*                                                                            */
/*                          ____   __  __    ___  ______                      */
/*                         |  _ \ | | | |  / __| |_   _|                      */
/*                         | |/ / | |_| |  \__ \   | |                        */
/*                         |_|\_\  \___/  |___/    |_|                        */
/*                                                                            */
/*   File:     main.rs                  Project:  toptui                      */
/*   Created:  2026-04-01               Updated:  2026-04-19                  */
/*   License:  MIT OR Apache-2.0                                              */
/*                                                                            */
/* ************************************************************************** */

mod process;

mod ui;
use ui::draw;

mod system;
use system::refresh;

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind,
};
use std::time::Duration;

use std::sync::{Arc, Mutex};
use std::thread;

use std::collections::HashMap;
use std::fs::{self};

fn read_uptime() -> Option<f64> {
    let uptime_file = fs::read_to_string("/proc/uptime").ok()?;
    let uptime = uptime_file.split_whitespace().next()?.parse::<f64>().ok()?;
    Some(uptime)
}

fn extract_uids_table() -> Option<Vec<(u32, String)>> {
    let mut uids_table: Vec<(u32, String)> = vec![];
    let etc_pass = fs::read_to_string("/etc/passwd").ok()?;
    for line in etc_pass.lines() {
        let Some(uid) = line.split(':').nth(2).and_then(|s| s.parse::<u32>().ok()) else {
            continue;
        };
        let user = line.split(':').next()?.to_string();
        uids_table.push((uid, user));
    }
    Some(uids_table)
}

const REFRESH_SECS: u64 = 2;

fn main() -> Result<(), ()> {
    let utime = read_uptime().ok_or(())?;
    let uids_table = extract_uids_table().ok_or(())?;
    let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) as f64 };

    let map = Arc::new(Mutex::new(HashMap::new()));

    let map_write = Arc::clone(&map);
    thread::spawn(move || -> Result<(), ()> {
        loop {
            if let Ok(mut map) = map_write.lock() {
                refresh(&mut map, utime, &uids_table, ticks, REFRESH_SECS as f64)?;
            }
            thread::sleep(Duration::from_secs(REFRESH_SECS));
        }
    });
    let map_read = Arc::clone(&map);
    let mut offset: usize = 0;
    crossterm::execute!(std::io::stdout(), EnableMouseCapture).map_err(|_| ())?;
    ratatui::run(|terminal| {
        loop {
            if event::poll(Duration::from_millis(250)).map_err(|_| ())? {
                let event = event::read().map_err(|_| ())?;
                if let Event::Key(key) = event
                    && key.code == KeyCode::Char('q')
                {
                    break Ok::<(), ()>(());
                }
                if let Event::Mouse(key) = event {
                    if key.kind == MouseEventKind::ScrollDown {
                        offset += 1
                    }
                    if key.kind == MouseEventKind::ScrollUp && offset > 0 {
                        offset -= 1
                    }
                }
            }
            if let Ok(map) = map_read.lock() {
                terminal
                    .draw(|frame| draw(frame, &map, offset))
                    .map_err(|_| ())?;
            }
        }
    })?;
    crossterm::execute!(std::io::stdout(), DisableMouseCapture).map_err(|_| ())?;
    Ok(())
}
