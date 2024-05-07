use libc::bsearch;
use nix::Error;
use crate::debugger_command::{DebuggerCommand};
use crate::inferior::{Inferior, Status};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use crate::dwarf_data::{DwarfData, Error as DwarfError, Line};

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    debug_data: DwarfData,
    break_point: Vec<usize>,
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };
        debug_data.print();
        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data,
            break_point: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    if self.inferior.is_some() {
                        self.inferior.as_mut().unwrap().kill();
                        self.inferior = None;
                    }
                    if let Some(inferior) = Inferior::new(&self.target, &args, &self.break_point) {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
                        // to the Inferior object
                        let status = self.inferior.as_mut().unwrap().continue_run(None).unwrap();
                        self.print_status(status);
                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Continue => {
                    match &self.inferior {
                        None => {
                            println!("Program is not running");
                        }
                        Some(x) => {
                            let status = self.inferior.as_mut().unwrap().continue_run(None).unwrap();
                            self.print_status(status);
                        }
                    }
                }
                DebuggerCommand::Backtrace => {
                    if self.inferior.is_none() {
                        println!("Program is not running");
                    } else {
                        self.inferior.as_mut().unwrap().print_backtrace(&self.debug_data).unwrap()
                    }
                }
                DebuggerCommand::Break(addr_str) => {
                    if addr_str.starts_with('*') {
                        let addr = parse_address(&addr_str[1..]);
                        match addr {
                            Some(addr_v) => {
                                if self.inferior.is_some() {
                                    match self.inferior.as_mut().unwrap().write_byte(addr_v, 0xcc) {
                                        Ok(_) => {}
                                        Err(_) => {
                                            println!("Add break point {:#x} failed", addr_v);
                                            continue;
                                        }
                                    }
                                }
                                println!("Set breakpoint {} at {:#x}", self.break_point.len(), addr_v);
                                self.break_point.push(addr_v);
                            }
                            None => println!("Invalid address"),
                        }
                    } else {
                        println!("Break need an address start with \'*\'")
                    }
                }
                DebuggerCommand::Quit => {
                    match &self.inferior {
                        None => {}
                        Some(_inferior) => {
                            self.inferior.as_mut().unwrap().kill();
                            self.inferior = None;
                        }
                    }
                    return;
                }
            }
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }

    pub fn print_status(&mut self, state: Status) {
        let status = self.inferior.as_mut().unwrap().continue_run(None).unwrap();
        Status::print(&status);
        match status {
            Status::Stopped(_signal, rip) => {
                let line_number = match self.debug_data.get_line_from_addr(rip) {
                    None => {}
                    Some(ln) => {
                        println!("Stopped at {}", ln);
                    }
                };

            }
            Status::Exited(_) => self.inferior = None,
            Status::Signaled(_) => self.inferior = None,
        }
    }
}

pub fn parse_address(addr: &str) -> Option<usize> {
    let addr_without_0x = if addr.to_lowercase().starts_with("0x") {
        &addr[2..]
    } else {
        &addr
    };
    usize::from_str_radix(addr_without_0x, 16).ok()
}