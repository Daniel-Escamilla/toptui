/* ************************************************************************** */
/*                                                                            */
/*                          ____   __  __    ___  ______                      */
/*                         |  _ \ | | | |  / __| |_   _|                      */
/*                         | |/ / | |_| |  \__ \   | |                        */
/*                         |_|\_\  \___/  |___/    |_|                        */
/*                                                                            */
/*   File:     main.rs                  Project:  toptui                      */
/*   Created:  2026-04-01               Updated:  2026-04-06                  */
/*   License:  MIT OR Apache-2.0                                              */
/*                                                                            */
/* ************************************************************************** */

mod process;

mod ui;
use ui::draw;

mod system;
use system::refresh;

use crossterm::event::{self, Event, KeyCode, EnableMouseCapture, MouseEventKind};
use std::time::Duration;

use std::sync::{Arc, Mutex};
use std::thread;

use std::collections::HashMap;
use std::fs::{self};

fn read_uptime() -> Option<f64> {
    let uptime_file = fs::read_to_string("/proc/uptime").ok()?;
    let uptime = uptime_file.split_whitespace().nth(0)?.parse::<f64>().ok()?;
    Some(uptime)
}

fn extract_uids_table() -> Option<Vec<(u32, String)>> {
    let mut uids_table: Vec<(u32, String)> = vec![];
    let etc_pass = fs::read_to_string("/etc/passwd").ok()?;
    for line in etc_pass.lines() {
        let Some(uid) = line.split(':').nth(2).and_then(|s| s.parse::<u32>().ok()) else {
            continue;
        };
        let user = line.split(':').nth(0)?.to_string();
        uids_table.push((uid, user));
    }
    Some(uids_table)
}

fn main() -> Result<(), ()> {
    let utime = read_uptime().ok_or(())?;
    let uids_table = extract_uids_table().ok_or(())?;

    let map = Arc::new(Mutex::new(HashMap::new()));

    let map_write = Arc::clone(&map);
    thread::spawn(move || -> Result<(), ()> {
        loop {
            if let Ok(mut map) = map_write.lock() {
                refresh(&mut map, utime, &uids_table)?;
            }
            thread::sleep(Duration::from_secs(2));
        }
    });
    let map_read = Arc::clone(&map);
    let mut offset: usize = 0;
    crossterm::execute!(std::io::stdout(), EnableMouseCapture).map_err(|_| ())?;
    ratatui::run(|terminal| {
        loop {
            if event::poll(Duration::from_millis(250)).map_err(|_| ())? {
                let event = event::read().map_err(|_| ())?;
                if let Event::Key(key) = event {
                    if key.code == KeyCode::Char('q') {
                        break Ok::<(), ()>(());
                    }
                }
                if let Event::Mouse(key) = event {
                    if key.kind == MouseEventKind::ScrollDown {
                        offset += 1
                    }
                    if key.kind == MouseEventKind::ScrollUp {
                        if offset > 0 { offset -= 1}
                    }
                }
            }
            if let Ok(map) = map_read.lock() {
                terminal.draw(|frame| draw(frame, &map, offset)).map_err(|_| ())?;
            }
        }
    })?;

    crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture).map_err(|_| ())?;
    Ok(())
}
