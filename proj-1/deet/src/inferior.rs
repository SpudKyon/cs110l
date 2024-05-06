use std::os::unix::process::CommandExt;
use nix::sys::ptrace;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::process::{Child, Command};
use libc::ptrace;
use nix::Error;
use crate::dwarf_data::DwarfData;

pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),

    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}

pub struct Inferior {
    child: Child,
}

impl Inferior {
    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(target: &str, args: &Vec<String>) -> Option<Inferior> {
        let child_process = unsafe {
            Command::new(target)
                .args(args)
                .pre_exec(child_traceme)
        }.spawn().ok()?;

        Some(Inferior { child: child_process })
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }

    pub fn continue_run(&self, signal: Option<signal::Signal>) -> Result<Status, nix::Error> {
        // When a process that has PTRACE_TRACEME enabled calls exec,
        // the operating system will load the specified program into the process,
        // and then (before the new program starts running) it will
        // pause the process using SIGTRAP. So we use ptrace::cont to wake up it
        let pid = self.pid();
        ptrace::cont(pid, signal)?;
        // dbg!("cont success");
        self.wait(None)
    }

    pub fn kill(&mut self) {
        self.child.kill().unwrap();
        self.wait(None).unwrap();
        println!("Killing running inferior (pid {})", self.pid())
    }

    pub fn print_backtrace(&self, debug_data: &DwarfData) -> Result<(), nix::Error> {
        let regs_struct = ptrace::getregs(self.pid())?;
        let mut instruction_pointer = regs_struct.rip as usize;
        let mut base_pointer = regs_struct.rbp as usize;
        loop {
            let func_name = debug_data.get_function_from_addr(instruction_pointer).unwrap();
            let line_number = debug_data.get_line_from_addr(instruction_pointer).unwrap();
            println!("{} ({})", func_name, line_number);
            if func_name == "main" {
                break;
            }
            instruction_pointer = ptrace::read(self.pid(), (base_pointer + 8) as ptrace::AddressType)? as usize;
            base_pointer = ptrace::read(self.pid(), base_pointer as ptrace::AddressType)? as  usize;
        }
        Ok(())
    }
}
