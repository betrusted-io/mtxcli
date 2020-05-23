//! Interactive Action
//!
//! Enters interactive mode

use lazy_static::lazy_static;
use std::error::Error;
use std::io::Write;
use std::process;
use std::sync::Mutex;
// use tokio::io::{AsyncReadExt, AsyncWriteExt };
use tokio::io::AsyncReadExt;
use tokio::io;
// use tokio::net::TcpStream;
use regex::Regex;

use crate::config::show;
use crate::config::url;

#[cfg(not(windows))]
use std::os::unix::io::RawFd;
#[cfg(not(windows))]
use nix::sys::signal;
#[cfg(not(windows))]
use nix::sys::termios::{self, SetArg, InputFlags, ControlFlags, LocalFlags};
#[cfg(not(windows))]
use nix::unistd;

use crate::config::Config;

/// Tab
const HT: u8 = 9;
/// Line Feed
const LF: u8 = 10;
/// Delete
const DEL: u8 = 127;
/// Control-C
const ETX: u8 = 3;
/// Buffer size constant
const MAX_BYTES: usize = 128;
#[cfg(not(windows))]
/// File descriptor for `STDIN`
const STDIN_FD: RawFd = 0;

#[cfg(not(windows))]
lazy_static! {
    /// Saved TERMIOS when `STDIN` is a TTY
    static ref PREVIOUS_TERMIOS: Mutex<Option<termios::Termios>> =
        Mutex::new(None);
}

/// Signal handler to restore the terminal state
///
/// Consult the global PREVIOUS_TERMIOS mutex.
/// If non-None, restore the previous termios.
/// This function MUST be called with all other signals masked.
///
/// # Panics
///   * If this signal handler is called while the PREVIOUS_TERMIOS is unlocked.
///     This cannot happen, as this handler is installed after the
///     PREVIOUS_TERMIOS, at which point it will not be updated again.
extern "C" fn restore_terminal(signal: libc::c_int) {
    if let Some(previous_termio) = PREVIOUS_TERMIOS.lock().unwrap().take() {
        println!("Restoring previous TERMIOS...");
        termios::tcsetattr(STDIN_FD, termios::SetArg::TCSANOW, &previous_termio)
            .expect("couldn't restore old flags");
    } else {
        println!("No previous TERMIOS to restore");
    }
    if signal > 0 {
        println!("terminating (from signal {})", signal);
        process::exit(signal);
    }
}

/// Will convert the TTY to 'raw' mode and save the original mode to `PREVIOUS_TERMIOS`
fn stdin_raw_mode() {
    println!("set raw terminal mode");
    let original_mode = termios::tcgetattr(STDIN_FD)
        .expect("Couldn't get original terminal mode");
    let mut raw_mode = original_mode.clone();
    *PREVIOUS_TERMIOS.lock().unwrap() = Some(original_mode);
    // disable BREAK interrupt, CR to NL conversion on input,
    // input parity check, strip high bit (bit 8), output flow control
    raw_mode.input_flags &= !(InputFlags::BRKINT
                              // | InputFlags::ICRNL
                              | InputFlags::INPCK
                              | InputFlags::ISTRIP
                              | InputFlags::IXON);
    // character-size mark (8 bits)
    raw_mode.control_flags |= ControlFlags::CS8;
    // disable echoing, canonical mode, extended input processing and signals
    raw_mode.local_flags &=
        !(LocalFlags::ECHO |
          LocalFlags::ICANON |
          LocalFlags::IEXTEN |
          LocalFlags::ISIG);
    termios::tcsetattr(STDIN_FD, SetArg::TCSAFLUSH, &raw_mode)
        .expect("Couldn't set terminal raw mode");
    // Install our signal handler for SIGINT and SIGTERM
    let sa = signal::SigAction::new(
        signal::SigHandler::Handler(restore_terminal),
        signal::SaFlags::empty(),
        signal::SigSet::all(),
    );
    unsafe {
        signal::sigaction(signal::Signal::SIGINT, &sa)
            .expect("Couldn't set signal handler for SIGINT");
        signal::sigaction(signal::Signal::SIGTERM, &sa)
            .expect("Couldn't set signal handler for SIGTERM");
    }
}

/// Returns true if `fd` is a TTY
///
/// Will assume false if unable to query the file descriptor
fn is_a_tty(fd: RawFd) -> bool {
    let ttyp = match unistd::isatty(fd) {
        Ok(ttyp) => ttyp,
        Err(e) => {
            log::warn!("unable to determine if fd={} is a tty, assuming false - use ^D to quit: {}", fd, e);
            return false;
        }
    };
    ttyp
}

/*
/// Prints received bytes as a UTF String (with newlines before and after)
fn received(n: usize, buf: &[u8]) {
    print!("\n{}", String::from_utf8((&buf[0..n]).to_vec())
           .expect("Couldn't convert received data from UTF8"));
}
 */


/// Print to stdout and flush
///
/// FFI: [https://github.com/rust-lang/rust/issues/23818](https://github.com/rust-lang/rust/issues/23818)
fn print_flush(s: &str) {
    print!("{}", s);
    std::io::stdout().flush().expect("Could not flush stdout");
}

/// prints help
fn help(_args: &str) {
    println!("-- mtxcli commands --");
    println!("/assign var=value -- assign variable (transient) ");
    println!("/decode value -- URL decode value");
    println!("/encode value -- URL encode value");
    println!("/get var -- get variable");
    println!("/help -- prints this help message");
    println!("/quit -- quit's mtxcli");
    println!("/set var=value -- set variable (saved)");
    println!("/show_config -- show configuration");
    println!("/unset var -- unset variable");
}

/// assign var=value
fn assign(config: &mut Config, args: &str) {
    let keqv = Regex::new(r"([^=]+)=([^=]+)").unwrap(); // TODO: cache this
    if let Some(cap) = keqv.captures(args) {
        config.assign(&config.eval(&cap[1]), &config.eval(&cap[2]));
    } else {
        println!("syntax error: /assign var=value");
    }
}

/// set var=value
fn set(config: &mut Config, args: &str) {
    let keqv = Regex::new(r"([^=]+)=([^=]+)").unwrap(); // TODO: cache this
    if let Some(cap) = keqv.captures(args) {
        config.set(&config.eval(&cap[1]), &config.eval(&cap[2]));
    } else {
        println!("syntax error: /assign var=value");
    }
}

/// get var
fn get(config: &mut Config, args: &str) {
    println!("{}", config.get(&config.eval(args)));
}

/// unset var
fn unset(config: &mut Config, args: &str) {
    config.unset(&config.eval(args));
}

/// Interprets line, returns true to quit
fn interpret(config: &mut Config, line: &str) -> bool {
    let re = Regex::new(r"/([a-z_\?]+)\s*(.*)?$") // TODO: cache this
        .expect("unable to compile interpret regex");
    if let Some(cap) = re.captures(line) {
        let command = &cap[1];
        let args = &cap[2];
        match command {
            "?" => help(args),
            "assign" => assign(config, args),
            "decode" => println!("{}", url::decode(args)),
            "encode" => println!("{}", url::encode(args)),
            "get" => get(config, args),
            "help" => help(args),
            "quit" => return true,
            "set" => set(config, args),
            "show_config" => {show::act(config); ()},
            "unset" => unset(config, args),
            _ => println!("invalid command: {}", command)
        }
    } else {
        println!("invalid command: {}", line);
    }
    false
}

#[allow(unreachable_code)]
#[tokio::main]
/// The main **chatcli** program
async fn interactive(config: &mut Config, _ttyp: bool) -> Result<(), Box<dyn Error>> {
    let name = "anonymous";
    let prompt = "[mtxcli] ";
    let mut sin = io::stdin();
    let mut inbuf = Box::new([0; MAX_BYTES]);
    let mut line: String = "".to_string();
    print_flush(prompt);
    loop {
        tokio::select! {
            /*
            netbytes = s_read.read(&mut netbuf[..]) => {
                let n = netbytes?;
                if n == 0 {
                    println!(" EOF"); break;
                } else {
                    received(n, &*netbuf);
                    if (*netbuf).starts_with(b"Please enter your username:") {
                        debug!("_connected_");
                        println!("> {}", name);
                        let _ = s_write.write((name.to_owned() + "\n").as_bytes()).await;
                    }
                    print_flush("> "); print_flush(&line);
                }
            }
             */
            inbytes = sin.read(&mut inbuf[..]) => {
                let n = inbytes?;
                if n == 0 {
                    println!(" EOF"); break
                } else {
                    let mut quit = false;
                    let mut i = 0;
                    while i < n {
                        let mut j = i;
                        while j < n && inbuf[j] >= 32 && inbuf[j] != DEL {
                            j += 1;
                        }
                        if j > i {
                            let word = String::from_utf8((&inbuf[i..j]).to_vec())
                                .expect("Couldn't convert typed data from UTF8");
                            line.push_str(&word);
                            print_flush(&word);
                        }
                        if j < n {
                            match inbuf[j] {
                                DEL => {
                                    if !line.is_empty() { // NOTE the subtlety: multi-byte
                                        line.pop(); // unicode chars are magically popped
                                        print_flush("\x08 \x08");
                                    }
                                },
                                ETX => { println!(" INT"); quit = true; break;},
                                HT => {
                                    let word = " ";
                                    line.push_str(word);
                                    print_flush(word);
                                },
                                LF => {
                                    println!();
                                    if line.starts_with('/') {
                                        quit = interpret(config, &line);
                                    } else {
                                        // if ttyp {
                                        println!("<{}> {}", name, line);
                                        // }
                                    }
                                    if !quit { print_flush(prompt); }
                                    line = "".to_string();
                                }
                                _ => {} // ignore other CTRL chars
                            }
                        }
                        i = j + 1;
                    }
                    if quit { break; }
                }
            }
        };
    }
    return Ok(());
}

/// Interactive Mode
pub fn act(config: &mut Config) -> i32  {
    println!("interactive mode");
    let ttyp = is_a_tty(STDIN_FD) && ! config.is("disable_tty");
    // let mut stream = TcpStream::connect(CHAT_SERVER).await?;
    // let (mut s_read, mut s_write) = stream.split();
    // let mut netbuf = Box::new([0; MAX_BYTES]);

    if ttyp {
        stdin_raw_mode();
    } else {
        warn!("not a TTY, use ^D to quit");
    }
    interactive(config, ttyp).unwrap_or_else(|e| { error!("interactive error: {:?}", e);});
    if ttyp {
        restore_terminal(0);
    }
    0
}
