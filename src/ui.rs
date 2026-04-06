/* ************************************************************************** */
/*                                                                            */
/*                          ____   __  __    ___  ______                      */
/*                         |  _ \ | | | |  / __| |_   _|                      */
/*                         | |/ / | |_| |  \__ \   | |                        */
/*                         |_|\_\  \___/  |___/    |_|                        */
/*                                                                            */
/*   File:     ui.rs                    Project:  proc-monitor                */
/*   Created:  2026-04-01               Updated:  2026-04-05                  */
/*   License:  MIT OR Apache-2.0                                              */
/*                                                                            */
/* ************************************************************************** */

use crate::process;
use process::Process;

use std::collections::HashMap;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
// use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph};

use Constraint::{Fill, Length, Min};

fn draw_grid(frame: &mut Frame, main_area: Rect, map: &HashMap<u32, Process>, offset: usize) {
    const BOX_WIDTH: u16 = 50;
    const BOX_HEIGHT: u16 = 6;

    let grid_columns = (main_area.width / BOX_WIDTH) + 1;
    let grid_rows = main_area.height / BOX_HEIGHT;

    let row_slots = vec![Length(BOX_HEIGHT); grid_rows as usize];

    let rows = Layout::vertical(row_slots).split(main_area);

    let mut process: Vec<&Process> = map.values().collect();
    process.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));

    let col_slots = vec![Fill(1); grid_columns as usize];

    let all_slots: Vec<_> = rows
        .iter()
        .flat_map(|row| Layout::horizontal(&col_slots).split(*row).to_vec())
        .collect();

    let n = (main_area.width / grid_columns - 7) as usize;

    all_slots
        .iter()
        .zip(process.iter().skip(offset * grid_columns as usize))
        .for_each(|(slot, process)| {
            let cmd1 = process.command.chars().take(n).collect::<String>();
            let cmd2 = process.command.chars().skip(n).take(n).collect::<String>();
            let info = format!(
                "NAME: {}\nRAM: {} KB\nCPU: {:.2} %\nCMD: {}\n     {}",
                process.name, process.memory_kb, process.cpu, cmd1, cmd2,
            );
            frame.render_widget(
                Paragraph::new(info).block(Block::bordered().title(format!(
                    "PID {}  {}",
                    process.pid.to_string(),
                    process.user
                ))),
                *slot,
            );
        });
}

fn draw_stats(frame: &mut Frame, stats_area: Rect) {
    frame.render_widget(
        Block::bordered()
            .border_style(Style::default().fg(Color::Cyan))
            .title("Stadistics Area"),
        stats_area,
    );
}

fn draw_filter(frame: &mut Frame, filter_area: Rect) {
    let horizontal_filter = Layout::horizontal([Fill(3), Fill(1)]);
    let [filters, search] = horizontal_filter.areas(filter_area);

    frame.render_widget(Block::bordered().title("Filters"), filters);
    frame.render_widget(Block::bordered().title("Search"), search);
}

pub fn draw(frame: &mut Frame, map: &HashMap<u32, Process>, offset: usize) {
    let vertical = Layout::vertical([Length(7), Length(3), Min(0)]);
    let [stats_area, filter_area, main_area] = vertical.areas(frame.area());

    draw_stats(frame, stats_area);
    draw_filter(frame, filter_area);
    draw_grid(frame, main_area, map, offset);
    // frame.render_widget(Block::bordered().title("Main Area"), main_area);
}
