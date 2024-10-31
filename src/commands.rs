use alloc::string::String;

use crate::println;

pub fn parse_command(command: String) {
    if command.trim().eq("test") { println!("Command processing up !") }
}