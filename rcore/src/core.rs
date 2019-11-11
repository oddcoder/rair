/*
 * core.rs: Linking all rair parts together into 1 module.
 * Copyright (C) 2019  Oddcoder
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use app_dirs::*;
use helper::*;
use io::{LISTMAPFUNCTION, MAPFUNCTION, PRINTHEXFUNCTION, UNMAPFUNCTION};
use loc::{MODEFUNCTION, SEEKFUNCTION};
use rio::*;
use rtrees::bktree::SpellTree;
use rustyline::Editor;
use std::io;
use std::io::Write;
use std::mem;
use std::path::PathBuf;
use writer::Writer;
use yansi::Paint;

pub struct Core {
    pub stdout: Writer,
    pub stderr: Writer,
    pub mode: AddrMode,
    pub io: RIO,
    pub rl: Editor<()>,
    loc: u64,
    app_info: AppInfo,
    commands: SpellTree<&'static CmdFunctions>,
    pub color_palette: Vec<(u8, u8, u8)>,
}
impl Default for Core {
    fn default() -> Self {
        Core {
            mode: AddrMode::Phy,
            stdout: Writer::new_write(Box::new(io::stdout())),
            stderr: Writer::new_write(Box::new(io::stderr())),
            io: RIO::new(),
            loc: 0,
            rl: Editor::<()>::new(),
            app_info: AppInfo { name: "rair", author: "RairDevs" },
            commands: SpellTree::new(),
            color_palette: Vec::new(),
        }
    }
}
impl Core {
    fn load_commands(&mut self) {
        self.add_command("map", &MAPFUNCTION);
        self.add_command("maps", &LISTMAPFUNCTION);
        self.add_command("mode", &MODEFUNCTION);
        self.add_command("m", &MODEFUNCTION);
        self.add_command("printHex", &PRINTHEXFUNCTION);
        self.add_command("px", &PRINTHEXFUNCTION);
        self.add_command("seek", &SEEKFUNCTION);
        self.add_command("s", &SEEKFUNCTION);
        self.add_command("unmap", &UNMAPFUNCTION);
        self.add_command("um", &UNMAPFUNCTION);
    }
    fn init_colors(&mut self) {
        self.color_palette.push((0x58, 0x68, 0x75));
        self.color_palette.push((0xb5, 0x89, 0x00));
        self.color_palette.push((0xcb, 0x4b, 0x16));
        self.color_palette.push((0xdc, 0x32, 0x2f));
        self.color_palette.push((0xd3, 0x36, 0x82));
        self.color_palette.push((0x6c, 0x71, 0xc4));
        self.color_palette.push((0x26, 0x8b, 0xd2));
        self.color_palette.push((0x2a, 0xa1, 0x98));
        self.color_palette.push((0x85, 0x99, 0x00));
    }
    pub fn new() -> Self {
        let mut core: Core = Default::default();
        core.load_commands();
        drop(core.rl.load_history(&core.hist_file()));
        core.init_colors();
        return core;
    }

    pub fn hist_file(&self) -> PathBuf {
        let mut history = app_dir(AppDataType::UserData, &self.app_info, "/").unwrap();
        history.push("history");
        return history;
    }

    pub fn set_loc(&mut self, loc: u64) {
        self.loc = loc;
    }

    pub fn get_loc(&self) -> u64 {
        self.loc
    }

    pub fn add_command(&mut self, command_name: &'static str, functionality: &'static CmdFunctions) {
        // first check that command_name doesn't exist
        let command = command_name.to_string();
        let (exact, _) = self.commands.find(&command, 0);
        if exact.is_empty() {
            self.commands.insert(command, functionality);
        } else {
            let msg = format!("Command {} already existed.", Paint::default(command_name).bold());
            error_msg(self, "Cannot add this command.", &msg);
        }
    }
    fn command_not_found(&mut self, command: &str) {
        let msg = format!("Command {} is not found.", Paint::default(command).bold());
        error_msg(self, "Execution failed", &msg);
        let (_, similar) = self.commands.find(&command.to_string(), 2);
        let mut s = similar.iter();
        if let Some(suggestion) = s.next() {
            let (r, g, b) = self.color_palette[5];
            write!(self.stderr, "Similar command: {}", Paint::rgb(r, g, b, suggestion)).unwrap();
            for suggestion in s {
                write!(self.stderr, ", {}", Paint::rgb(r, g, b, suggestion)).unwrap();
            }
            writeln!(self.stderr, ".").unwrap();
        }
    }

    pub fn run(&mut self, command: &str, args: &[String]) {
        let (exact, _) = self.commands.find(&command.to_string(), 2);
        if exact.is_empty() {
            self.command_not_found(command);
        } else {
            (exact[0].run)(self, args)
        }
    }

    pub fn run_at(&mut self, command: &str, args: &[String], at: u64) {
        let old_loc = mem::replace(&mut self.loc, at);
        self.run(command, args);
        self.loc = old_loc;
    }

    pub fn help(&mut self, command: &str) {
        let (exact, _) = self.commands.find(&command.to_string(), 2);
        if exact.is_empty() {
            self.command_not_found(command);
        } else {
            (exact[0].help)(self);
        }
    }
}

#[cfg(test)]
mod test_core {
    use super::*;
    #[test]
    fn test_loc() {
        let mut core = Core::new();
        core.set_loc(0x500);
        assert_eq!(core.get_loc(), 0x500);
    }
    #[test]
    fn test_add_command() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.add_command("a_non_existing_command", &SEEKFUNCTION);
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.add_command("s", &SEEKFUNCTION);
        assert_eq!(core.stderr.utf8_string().unwrap(), "Error: Cannot add this command.\nCommand s already existed.\n");
    }
    #[test]
    fn test_help() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.help("seeker");
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "Error: Execution failed\nCommand seeker is not found.\nSimilar command: seek.\n");
    }
    #[test]
    fn test_run_at() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.run_at("seeker", &[], 0x500);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "Error: Execution failed\nCommand seeker is not found.\nSimilar command: seek.\n");
    }
}