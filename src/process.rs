/* ************************************************************************** */
/*                                                                            */
/*                          ____   __  __    ___  ______                      */
/*                         |  _ \ | | | |  / __| |_   _|                      */
/*                         | |/ / | |_| |  \__ \   | |                        */
/*                         |_|\_\  \___/  |___/    |_|                        */
/*                                                                            */
/*   File:     process.rs               Project:  toptui                      */
/*   Created:  2026-04-01               Updated:  2026-04-06                  */
/*   License:  MIT OR Apache-2.0                                              */
/*                                                                            */
/* ************************************************************************** */

use std::fmt;
use std::fs::{self};

#[derive(Default)]
pub enum Status {
    Running,
    Sleeping,
    Stopped,
    Zombie,
    DiskSleep,
    Idle,
    #[default]
    Unknown,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Running => write!(f, "Running"),
            Status::Sleeping => write!(f, "Sleeping"),
            Status::Stopped => write!(f, "Stopped"),
            Status::Zombie => write!(f, "Zombie"),
            Status::DiskSleep => write!(f, "Disk Sleep"),
            Status::Idle => write!(f, "Idle"),
            Status::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Default)]
pub struct Process {
    pub pid: u32,
    pub name: String,
    pub state: Status,
    pub memory_kb: u64,
    pub virt_kb: u64,
    pub shr_kb: u64,
    pub threads: u32,
    pub user: String,
    pub command: String,
    pub utime: u64,
    pub stime: u64,
    pub prev_utime: u64,
    pub prev_stime: u64,
    pub cpu: f64,
    pub time: (u64, u64, u64, u64),
}

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "PID:     {}", self.pid)?;
        writeln!(f, "User:    {}", self.user)?;
        writeln!(f, "Name:    {}", self.name)?;
        writeln!(f, "State:   {}", self.state)?;
        writeln!(f, "Memory:  {} KB", self.memory_kb)?;
        writeln!(f, "Virt:    {} KB", self.virt_kb)?;
        writeln!(f, "Shr:     {} KB", self.shr_kb)?;
        writeln!(f, "Threads: {}", self.threads)?;
        writeln!(f, "Command: {}", self.command)?;
        writeln!(f, "Utime:   {}", self.utime)?;
        writeln!(f, "Stime:   {}", self.stime)?;
        writeln!(
            f,
            "Time:    {}:{}:{}.{}",
            self.time.0, self.time.1, self.time.2, self.time.3
        )?;
        write!(f, "CPU:     {} %", self.cpu)
    }
}

fn choose_status(status: &str) -> Status {
    match status {
        "R" => Status::Running,
        "S" => Status::Sleeping,
        "T" => Status::Stopped,
        "Z" => Status::Zombie,
        "D" => Status::DiskSleep,
        "I" => Status::Idle,
        _ => Status::Unknown,
    }
}

fn choose_user(uid: u32, uids_table: &[(u32, String)]) -> Option<String> {
    uids_table
        .iter()
        .find_map(|(id, user)| (*id == uid).then_some(user.clone()))
}

fn get_time(seconds: f64) -> (u64, u64, u64, u64) {
    let total_ms = (seconds * 1000.0) as u64;
    let ms = total_ms % 1000;
    let total_s = total_ms / 1000;
    let secs = total_s % 60;
    let mins = (total_s / 60) % 60;
    let hours = total_s / 3600;
    (hours, mins, secs, ms)
}

impl Process {
    const TICKS_PER_SECOND: f64 = 200.0;

    pub fn new(pid: u32, utime: f64, uids_table: &[(u32, String)]) -> Result<Self, ()> {
        let mut process = Process {
            pid,
            ..Default::default()
        };
        process.read_status(uids_table)?;
        process.read_cmdline()?;
        process.read_stat(utime)?;
        Ok(process)
    }

    fn read_status(&mut self, uids_table: &[(u32, String)]) -> Result<(), ()> {
        let status_file =
            fs::read_to_string(format!("/proc/{}/status", self.pid)).map_err(|_| ())?;
        for line in status_file.lines() {
            let Some(category) = line.split(':').nth(0) else {
                continue;
            };
            let category = category.trim();
            let Some(value) = line.split_whitespace().nth(1) else {
                continue;
            };
            match category {
                "Name" => self.name = value.to_string(),
                "State" => self.state = choose_status(value),
                "VmSize" => self.virt_kb = value.parse::<u64>().map_err(|_| ())?,
                "VmRSS" => self.memory_kb = value.parse::<u64>().map_err(|_| ())?,
                "RssShmem" => self.shr_kb = value.parse::<u64>().map_err(|_| ())?,
                "Threads" => self.threads = value.parse::<u32>().map_err(|_| ())?,
                "Uid" => {
                    self.user =
                        choose_user(value.parse::<u32>().map_err(|_| ())?, uids_table).ok_or(())?
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn read_stat(&mut self, utime: f64) -> Result<(), ()> {
        let stat_file = fs::read_to_string(format!("/proc/{}/stat", self.pid)).map_err(|_| ())?;
        let values: Vec<_> = stat_file.split_whitespace().collect();
        self.utime = values.get(13).ok_or(())?.parse::<u64>().map_err(|_| ())?;
        self.stime = values.get(14).ok_or(())?.parse::<u64>().map_err(|_| ())?;

        if self.prev_utime != 0 {
            self.cpu = (self.utime + self.stime - self.prev_utime - self.prev_stime) as f64
                / Self::TICKS_PER_SECOND
                * 100.0;
        }
        self.prev_utime = self.utime;
        self.prev_stime = self.stime;

        let starttime = values.get(21).ok_or(())?.parse::<f64>().map_err(|_| ())?;
        self.time = get_time(utime - (starttime / 100.0));
        Ok(())
    }

    fn read_cmdline(&mut self) -> Result<(), ()> {
        let cmdline_file =
            fs::read_to_string(format!("/proc/{}/cmdline", self.pid)).map_err(|_| ())?;
        self.command = cmdline_file.replace("\0", " ");
        Ok(())
    }
}

pub fn get_pids() -> Vec<u32> {
    let Ok(entries) = fs::read_dir("/proc") else {
        return vec![];
    };
    entries
        .filter_map(|directory| {
            let entry = directory.ok()?;
            let name = entry.file_name();
            let name_str = name.to_str()?.to_string();
            name_str.parse::<u32>().ok()
        })
        .collect()
}
