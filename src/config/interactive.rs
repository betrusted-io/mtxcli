//! Interactive Action
//!
//! Enters interactive mode

use std::convert::TryInto;
// use std::error::Error;
use std::io::Write;
use tokio::io::AsyncReadExt;
use tokio::io;
use regex::Regex;
use tokio::time::{self, Duration,Instant,Sleep};
use unicode_width::UnicodeWidthStr;
// use tokio::sync::mpsc;
use hyper_tls::HttpsConnector;
use std::collections::HashMap;

type OpMap = HashMap<String,String>;

const CONTENT_TYPE: &str = "Content-Type";

fn web_request(url: String) -> OpMap {
    let mut r = OpMap::new();
    r.insert("op".to_string(), "web".to_string());
    r.insert("url".to_string(), url);
    r
}

/*
use std::fmt;

#[derive(Debug)]
struct CustomError {
    // msg: String
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "CustomeError: {}", self.msg)
        write!(f, "CustomeError")
    }
}

impl Error for CustomError {}

impl Error for CustomError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.msg)
    }
}
*/
/*
use tokio::io::{self, AsyncReadExt, AsyncWriteExt as _};
 */

/* RECENT
*/
// use hyper::client::ResponseFuture;
// use hyper::{Result, Response, Body};
use hyper::Result;
use hyper::{body::HttpBody as _, Client};

#[cfg(not(windows))]
use std::sync::Mutex;
#[cfg(not(windows))]
use std::process;
#[cfg(not(windows))]
use lazy_static::lazy_static;
#[cfg(not(windows))]
use std::os::unix::io::RawFd;
#[cfg(not(windows))]
use nix::sys::signal;
#[cfg(not(windows))]
use nix::sys::termios::{self, SetArg, InputFlags, ControlFlags, LocalFlags};
#[cfg(not(windows))]
use nix::unistd;

use crate::config::matrix;
use crate::config::show;
use crate::config::url;
use crate::config::datetime;
use crate::config::Config;

/// The key for the clock delay
const CLOCK_KEY: &str = "clock_delay";
/// Maximum delay
const CLOCK_MAX: i64 = 1_000_000_000;

/// Start of Heading (^A)
const SOH: u8 = 1;
/// Start of Text (^B)
const STX: u8 = 2;
/// End of text (^C)
const ETX: u8 = 3;
/// End of transmission (^D)
const EOT: u8 = 4;
/// Enquiry (^E)
const ENQ: u8 = 5;
/// Acknowledge (^F)
const ACK: u8 = 6;
/// Tab (^I)
const HT: u8 = 9;
/// Line Feed (^J)
const LF: u8 = 10;
/// Vertical tab (^K)
const VT: u8 = 11;
/// Form Feed (^L)
const FF: u8 = 12;
/// (^P)
const DLE: u8 = 16;
/// Escape ^[
const ESC: u8 = 27;
/// Delete
const DEL: u8 = 127;

/// Buffer size constant
const MAX_BYTES: usize = 128;
#[cfg(not(windows))]
/// File descriptor for `STDIN`
const STDIN_FD: RawFd = 0;
#[cfg(windows)]
/// File descriptor for `STDIN`
const STDIN_FD: u32 = 0;

// Previous terminal settings (TERMIOS)
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
#[cfg(not(windows))]
extern "C" fn restore_terminal(signal: libc::c_int) {
    if let Some(previous_termio) = PREVIOUS_TERMIOS.lock().unwrap().take() {
        // println!("Restoring previous TERMIOS...");
        termios::tcsetattr(STDIN_FD, termios::SetArg::TCSANOW, &previous_termio)
            .expect("couldn't restore old flags");
    }/* else {
        println!("No previous TERMIOS to restore");
    }*/
    if signal > 0 {
        println!("terminating (from signal {})", signal);
        process::exit(signal);
    }
}
#[cfg(windows)]
/// Signal handler to restore the terminal state
fn restore_terminal(_signal: u32) {
}

/// Will convert the TTY to 'raw' mode and save the original mode to
/// `PREVIOUS_TERMIOS`
#[cfg(not(windows))]
fn stdin_raw_mode() {
    // println!("set raw terminal mode");
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
#[cfg(windows)]
fn stdin_raw_mode() {
}

#[cfg(not(windows))]
/// Returns true if `fd` is a TTY
/// Will assume false if unable to query the file descriptor
fn is_a_tty(fd: RawFd) -> bool {
    match unistd::isatty(fd) {
        Ok(ttyp) => ttyp,
        Err(e) => {
            log::warn!("unable to determine if fd={} is a tty, assuming false - use ^D to quit: {}", fd, e);
            false
        }
    }
}

#[cfg(windows)]
/// Returns true if `fd` is a TTY
fn is_a_tty(_fd: u32) -> bool {
    false
}

/// Print to stdout and flush
/// FFI: [https://github.com/rust-lang/rust/issues/23818](https://github.com/rust-lang/rust/issues/23818)
fn output(s: &str) {
    std::io::stdout().write_all(s.as_bytes())
        .expect("could not write to stdout");
    std::io::stdout().flush()
        .expect("Could not flush stdout");
}

/*
/// Print integer value of a char (for debugging)
fn debug_char(ch: u8) {
    let mut d = String::new();
    d.push('<');
    d.push_str(&ch.to_string());
    d.push('>');
    output(&d);
}
 */

/// Print hex values of a raw string (for debugging)
#[allow(clippy::needless_range_loop)]
fn debug_buf(buf: &[u8], n: usize) -> String {
    let mut d = String::new();
    d.push('[');
    for i in 0..n {
        if i > 0 {
            d.push(',');
        }
        let hex = format!("{:02x}", buf[i]);
        d.push_str(&hex); // hex
    }
    d.push(']');
    d
}

/// Prints help
fn help(_args: &str) {
    println!("-- mtxcli commands --");
    println!("/assign var=value -- assign variable (transient) ");
    println!("/clock -- show the world time");
    println!("/decode value -- URL decode value");
    println!("/encode value -- URL encode value");
    println!("/get var -- get variable");
    println!("/help -- prints this help message");
    println!("/login_types [server]-- shows supported login types for server");
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

/// get login_types
fn login_types(config: &mut Config, args: &str) {
    let user = config.get(&config.eval("user"));
    let server = if args.is_empty() {
        matrix::get_server(&user)
    } else {
        args
    };
    config.set("_URL_", &matrix::login_types(server));
}

/// returns true to reset clock interval
fn clock(config: &mut Config, args: &str) -> bool {
    if args.is_empty() {
        println!("{}", &datetime::now_world());
        return false;
    } else {
        let delay_ms: i64 = if args == "off" {
            CLOCK_MAX
        } else if let Ok(d) = args.parse::<i64>() {
            d
        } else {
            println!("invalid argument, /clock {{off|millis}}");
            CLOCK_MAX
        };
        config.set_integer(CLOCK_KEY, delay_ms);
    }
    true
}

/// Action for the program to act upon
#[derive(Debug, PartialEq)]
enum InputAction {
    /// Continue processing input
    Continue,
    /// Quit
    Quit,
    /// ResetDelay
    ResetDelay,
    /// MakeRequest
    MakeRequest
}

/// Interprets line, returns command
#[allow(clippy::unused_unit)]
fn interpret(config: &mut Config, line: &str) -> InputAction {
    let re = Regex::new(r"/([a-z_\?]+)\s*(.*)?$") // TODO: cache this
        .expect("unable to compile interpret regex");
    if let Some(cap) = re.captures(line) {
        let command = &cap[1];
        let args = &cap[2];
        match command {
            "?" => help(args),
            "assign" => assign(config, args),
            "clock" => {
                if clock(config, args) {
                    return InputAction::ResetDelay;
                }
            },
            "decode" => println!("{}", url::decode(args)),
            "encode" => println!("{}", url::encode(args)),
            "get" => get(config, args),
            "help" => help(args),
            "login_types" => { login_types(config, args);
                               return InputAction::MakeRequest;
            },
            "quit" => return InputAction::Quit,
            "set" => set(config, args),
            "show_config" => { show::act(config); () },
            "unset" => unset(config, args),
            _ => println!("invalid command: {}", command)
        }
    } else {
        println!("invalid command: {}", line);
    }
    InputAction::Continue
}

/// Interactive struct
#[derive(Debug)]
pub struct Interactive<'b> {
    config: &'b mut Config<'b>,
    name: &'b str,
    room: &'b str,
    previous_line: String,
    inbuf: Box<[u8]>,
    uni_chars: Vec<String>,
    uni_lens: Vec<usize>,
    cursor: usize,
    line: String,
    line_cursor: usize,
    ttyp: bool,
    counter: u32,
    requests: Vec<OpMap>
}

/// implementation of Interactive
impl<'b> Interactive<'b> {

    /// Construct a new Interactive
    pub fn new(config: &'b mut Config<'b>, ttyp: bool) -> Self {
        let name = "anonymous";
        let room = config.app;
        let previous_line = "".to_string();
        let inbuf = Box::new([0; MAX_BYTES]);
        let uni_chars: Vec<String> = Vec::new();
        let uni_lens: Vec<usize> = Vec::new();
        let cursor = 0;
        let line = "".to_string();
        let line_cursor = 0;
        let counter = 0;
        let requests = Vec::new();
        Interactive {
            config,
            name,
            room,
            previous_line,
            inbuf,
            uni_chars,
            uni_lens,
            cursor,
            line,
            line_cursor,
            ttyp,
            counter,
            requests
        }
    }

    /// Print the current command line (and place the cursor appropriately)
    fn prompt(&self) {
        let mut p = String::new();
        p.push('[');
        p.push_str(self.room);
        p.push(']');
        p.push(' ');
        p.push_str(&self.line);
        output(&p);
        let mut erase = self.uni_chars.len();
        while erase > self.cursor {
            erase -= 1;
            for _ in 0..self.uni_chars[erase].width() {
                output("\x08");
            }
        }
    }

    /// Move the cursor to the beginning of the line and reprint it
    fn redraw_line(&self, max: usize) {
        let erase = max - self.cursor + 8;
        let n = 3 + self.room.len() + self.line.len() + erase;
        for _ in 0..erase {
            output(" ");
        }
        for _ in 0..n {
            output("\x08");
        }
        self.prompt();
    }

    /// Start a new delay timer to show the world clock
    fn new_delay(&self) -> Sleep {
        let delay_ms: u64 = self.config.get_default_int(CLOCK_KEY, CLOCK_MAX).try_into().unwrap();
        time::sleep(Duration::from_millis(delay_ms))
    }

    /// When the delay is up print the world time
    fn handle_delay(&self) -> Sleep {
        println!();
        println!("{}", &datetime::now_world());
        self.prompt();
        self.new_delay()
    }

    /// Print detailed information about the current command line
    fn debug_cursor(&self, msg: &str) {
        debug!("{} CURSOR={}:{:?},{:?}\nLINE={}={}=", msg, self.cursor,
               self.uni_chars, self.uni_lens,
               self.line_cursor, self.line);
    }

    /// Handle unicode input (break into unicode chars)
    fn handle_unicode(&mut self, i: usize, j: usize) {
        let mut uni = match String::from_utf8((&self.inbuf[i..j]).to_vec()) {
            Ok(u) => u,
            Err(e) => {
                error!("unable to convert input to UTF-8: {:?}", e);
                "".to_string()
            }
        };
        let mut uni_len = uni.len();
        if uni_len == 1 { // NOT a multi-byte char
            output(&uni);
            if self.cursor == self.uni_lens.len() {
                self.line.push_str(&uni);
                self.uni_chars.push(uni);
                self.uni_lens.push(1);
            } else {
                self.line.insert_str(self.line_cursor, &uni);
                self.uni_chars.insert(self.cursor, uni);
                self.uni_lens.insert(self.cursor, 1);
            }
            self.cursor += 1;
            self.line_cursor += 1;
            debug!("added 1");
        } else { // 1+ single/multi byte char
            let mut new_cursor = self.cursor;
            let mut new_line_cursor = self.line_cursor;
            let mut jj = j;
            let mut coalesce = false;
            while !uni.is_empty() {
                self.debug_cursor("multi");
                let u_ch = uni.pop().unwrap(); // one unicode char
                let length = uni_len - uni.len();
                let ii = jj - length;
                let u_str = u_ch.to_string();
                let next_coalesce = u_str == "\u{fe0f}" ||
                    u_str == "\u{200d}"; // ||
                debug!("u_ch={}={}= NEXT COALESCE={}", u_ch, u_str,next_coalesce);

                self.line.insert_str(self.line_cursor, &u_str);
                new_line_cursor += length;
                if coalesce {
                    let mut combined = self.uni_chars[self.cursor].clone();
                    combined.insert_str(0, &u_str);
                    self.uni_chars[self.cursor] = combined;
                    self.uni_lens[self.cursor] += length;
                } else {
                    self.uni_chars.insert(self.cursor, u_str);
                    new_cursor += 1;
                    self.uni_lens.insert(self.cursor, length);
                }
                coalesce = next_coalesce;
                jj = ii;
                uni_len = uni.len();
                debug!("MULTIPLE new_cursor={}, uni_len={}",
                       new_cursor, uni_len);
            }
            self.cursor = new_cursor;
            self.line_cursor = new_line_cursor;
        }
        self.debug_cursor("input");
        if self.cursor < self.uni_lens.len() || uni_len == 0 {
            debug!("redrawing b/c not at end of line");
            self.redraw_line(self.uni_chars.len());
        }
    }

    /// Move cursor backward one char
    fn backward_char(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            let length = self.uni_lens[self.cursor];
            let w = self.uni_chars[self.cursor].width();
            debug!("BACK UP length={}, w={} ch={}=", length, w,&self.uni_chars[self.cursor]);
            for _ in 0..w {
                output("\x08");
            }
            self.line_cursor -= length;
        }
        self.debug_cursor("STX");
    }

    /// Move cursor forward one char
    fn forward_char(&mut self) {
        if self.cursor < self.uni_chars.len() {
            output(&self.uni_chars[self.cursor]);
            self.line_cursor += self.uni_lens[self.cursor];
            self.cursor += 1;
        }
        self.debug_cursor("ACK");
    }

    /// Delete previous char
    fn delete_backward_char(&mut self) {
        if self.cursor > 0 {
            if self.cursor == self.uni_chars.len() {
                self.cursor -= 1;
                self.uni_chars.pop();
                let length = self.uni_lens.pop().unwrap();
                self.line_cursor -= length;
                self.line.pop(); // unicode
                if length == 1 {
                    output("\x08 \x08");
                } else {
                    debug!("redrawing b/c multi-byte");
                    self.redraw_line(self.uni_chars.len());
                }
            } else {
                self.cursor -= 1;
                let length = self.uni_lens[self.cursor];
                self.uni_chars.remove(self.cursor);
                self.uni_lens.remove(self.cursor);
                self.line_cursor = 0;
                for i in 0..self.cursor {
                    self.line_cursor += self.uni_lens[i];
                }
                let n = self.line_cursor + length;
                self.line.drain(self.line_cursor..n);
                debug!("redrawing b/c DEL not at end");
                self.redraw_line(self.uni_chars.len());
            }
        }
        self.debug_cursor("DEL");
    }

    /// Delete char at cursor
    fn delete_char(&mut self) {
        if self.cursor < self.uni_chars.len() {
            let length = self.uni_lens[self.cursor];
            self.uni_chars.remove(self.cursor);
            self.uni_lens.remove(self.cursor);
            let n = self.line_cursor + length;
            self.line.drain(self.line_cursor..n);
            debug!("redrawing b/c ^D");
            self.redraw_line(self.uni_chars.len());
        }
        self.debug_cursor("EOT");
    }

    /// Delete until the end of line
    fn kill_line(&mut self) {
        if self.cursor < self.uni_chars.len() {
            let max = self.uni_chars.len();
            self.uni_chars.drain(self.cursor..);
            self.uni_lens.drain(self.cursor..);
            self.line.drain(self.line_cursor..);
            debug!("redrawing b/c ^D");
            self.redraw_line(max);
        }
        self.debug_cursor("EOT");
    }

    /// Move cursor to the beginning of the line
    fn move_beginning_of_line(&mut self) {
        if self.cursor > 0 {
            for i in 1..(self.cursor+1) {
                let w = self.uni_chars[self.cursor - i].width();
                debug!("BACK i={}, w={}, ch={}",
                       self.cursor-i,w,&self.uni_chars[self.cursor - i]);
                for _ in 0..w {
                    output("\x08");
                }
            }
            self.cursor = 0;
            self.line_cursor = 0;
        }
        self.debug_cursor("SOH");
    }

    /// Move cursor to the end of the line
    fn move_end_of_line(&mut self) {
        while self.cursor < self.uni_chars.len() {
            output(&self.uni_chars[self.cursor]);
            self.line_cursor += self.uni_lens[self.cursor];
            self.cursor += 1;
        }
        self.debug_cursor("ENQ");
    }

    /// Move backwards one char
    fn previous_line(&mut self) {
        debug!("previous_line");
        self.uni_chars.clear();
        self.uni_lens.clear();
        self.line.clear();
        self.cursor = 0;
        self.line_cursor = 0;
        self.inbuf = Box::from(self.previous_line.as_bytes());
        self.handle_unicode(0, self.previous_line.len());
        self.move_end_of_line();
    }

    /// User pressed Return, execute the action (or send the message)
    fn execute(&mut self) -> InputAction {
        println!();
        self.debug_cursor("LF");
        let mut action = InputAction::Continue;
        if self.line.starts_with('/') {
            action = interpret(self.config, &self.line);
        } else {
            println!("<{}> {}", self.name, self.line);
        }
        self.previous_line = self.line.clone();
        self.uni_chars.clear();
        self.uni_lens.clear();
        self.line.clear();
        self.cursor = 0;
        self.line_cursor = 0;
        if action == InputAction::Continue {
            self.prompt();
        }
        action
    }

    /// Handle input from the keyboard
    fn handle_input(&mut self, n: usize) -> InputAction {
        if n == 0 {
            println!(" EOF");
            return InputAction::Quit;
        } else {
            let mut i = 0;
            debug!("RAW={}", debug_buf(&self.inbuf, n));
            while i < n {
                let mut j = i;
                while j < n && self.inbuf[j] >= 32 && self.inbuf[j] != DEL {
                    j += 1;
                }
                debug!("INPUT: i={}, j={}, n={}", i, j, n);
                if j > i { // add uni[code] symbol to uni_chars, len to uni_lens
                    self.handle_unicode(i, j);
                }
                if j < n { // handle control char
                    match self.inbuf[j] {
                        LF => { let action = self.execute(); // Enter
                                if action !=  InputAction::Continue {
                                    return action;
                                }
                        },
                        DEL => self.delete_backward_char(),
                        EOT => self.delete_char(), // ^D
                        SOH => self.move_beginning_of_line(), // ^A
                        STX => self.backward_char(), // ^B
                        ACK => self.forward_char(), // ^F
                        ENQ => self.move_end_of_line(), // ^E
                        VT => self.kill_line(), // ^K
                        FF => self.redraw_line(self.uni_chars.len()), // ^L
                        HT => { let uni = " "; // Tab
                                self.line.push_str(uni);
                                output(uni);
                        },
                        DLE => { // ^P
                            self.previous_line();
                            return InputAction::Continue;
                        },
                        ESC => {
                            debug!("ESC begin");
                            if j <= n + 2 && self.inbuf[j+1] == 91 {
                                j += 2;
                                debug!("ESC 91 = {}", self.inbuf[j]);
                                match self.inbuf[j] {
                                    65 => {
                                        self.previous_line();
                                        return InputAction::Continue;
                                    },
                                    66 => debug!("down"),
                                    67 => self.forward_char(),
                                    68 => self.backward_char(),
                                    _ => j -= 2 // ignore ESC
                                }
                            }
                        },
                        ETX => { println!(" INT"); // ^C
                                 return InputAction::Quit;
                        },
                        _ => {} // debug_char(self.inbuf[j])
                    } // match
                }
                i = j + 1;
            } // while
        }
        InputAction::Continue
    }

    /*
    async fn request(&mut self, input: Option<String>) -> Option<String> {
        println!("\nREQUEST: {:?}", input);
        let url_str = match input {
            Some(input) => input,
            None => return None
        };
        // time::sleep(Duration::from_millis(5000)).await;
        let url = url_str.parse::<hyper::Uri>().unwrap();
        if url.scheme_str() != Some("http") {
            println!("This example only works with 'http' URLs.");
        }
        let client = Client::new();
        println!("client: {:?}, url: {}", client, url);
        let responser = client.get(url).await;
        match responser {
            Ok(mut response) => {
                println!("Response: {}", response.status());
                println!("Headers: {:#?}\n", response.headers());
                let mut res = String::new();
                while let Some(next) = response.data().await {
                    let chunk = next.unwrap();
                    match String::from_utf8(chunk.to_vec()) {
                        Ok(str) => {
                            res.push_str(&str);
                        },
                        Err(e) => {
                            error!("unable to convert chunk to UTF-8: {:?}", e);
                        }
                    }
                }
                return Some(res);
            },
            Err(e) => {
                error!("could not get response: {:}", e);
                return None;
            }
        }
    }
     */

    fn handle_response(&mut self, response: Option<String>) {
        // println!("\nRESPONSE: {:?}", response);
        println!();
        match response {
            Some(res) => {
                println!("RESPONSE: {}, counter: {}", res, self.counter);
            }
            None => {
                println!("RESPONSE: None");
            }
        }
        self.prompt();
    }

    #[allow(unreachable_code)]
    #[tokio::main]
    /// Main asynchronous loop
    // async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    async fn run(&mut self) -> Result<()> {
        let mut sin = io::stdin();
        let mut delay = self.new_delay();
        // let (mut tx, mut rx) = mpsc::channel(32);
        let mut requesting = false;
        // let client = Client::new();
        let operation = request(None);
        // let operation = request(client, None);
        // let operation = self.request(None);
        tokio::pin!(operation);

        self.prompt();
        loop {
            tokio::select! {
                inbytesr = sin.read(&mut self.inbuf[..]) => {
                    match inbytesr {
                        Ok(inbytes) => match self.handle_input(inbytes) {
                            InputAction::Quit => break,
                            InputAction::ResetDelay => delay.reset(Instant::now()),
                            InputAction::MakeRequest => {
                                self.requests.push(web_request(self.config.get("_URL_")));
                                /*
                                match tx.send(self.config.get("_URL_")).await {
                                    Ok(_) => (),
                                    Err(e) => {
                                        error!("could tx: {:}", e);
                                    }
                                }
                                 */
                            },
                            _ => ()
                        },
                        Err(e) => {
                            error!("could not read standard input: {:}", e);
                            return Ok(());
                        }
                    }
                },
                response = &mut operation, if requesting => {
                    // handle_response(response);
                    self.handle_response(response);
                    requesting = false;
                },
                /*
                Some(rxmsg) = rx.recv(), if !requesting  => {
                    self.counter += 1;
                    // println!("RXMSG {:?}, counter: {}", rxmsg, self.counter);
                    operation.set(request(Some(rxmsg)));
                    // operation.set(request(client, Some(rxmsg)));
                    // operation.set(self.request(Some(rxmsg)));
                    requesting = true;
                }
                 */
                _ = &mut delay => delay = self.handle_delay()
            };
            if !requesting && !self.requests.is_empty() {
                let m = self.requests.remove(0);
                match m.get("op") {
                    Some(op) => {
                        debug!("request op = {}", op);
                        match op.as_str() {
                            "web" => {
                                let url = m.get("url").expect("expected url");
                                debug!("web url = {}", url);
                                self.counter += 1;
                                operation.set(request(Some(url.to_string())));
                                requesting = true;
                            },
                            _ => {
                                debug!("request op = {} unknown", op);
                            }
                        }
                    },
                    None => {
                        debug!("request has no op?");
                    }
                }
            }
        }
        return Ok(());
    }
}

// async fn request(client: Client, input: Option<String>) -> Option<String> {
async fn request(input: Option<String>) -> Option<String> {
    // println!("\nREQUEST: {:?}", input);
    let url_str = match input {
        Some(input) => input,
        None => return None
    };
    // time::sleep(Duration::from_millis(5000)).await;
    let url = url_str.parse::<hyper::Uri>().unwrap();
    /*
    if url.scheme_str() != Some("http") {
    println!("This example only works with 'http' URLs.");
}
    let client = Client::new();
     */
    match url.scheme_str() {
        Some("http") => {
            let client = Client::new();
            // println!("client: {:?}, url: {}", client, url);
            match client.get(url).await {
                Ok(mut response) => {
                    // println!("Response: {}", response.status());
                    // println!("Headers: {:#?}\n", response.headers());
                    let mut res = String::new();
                    while let Some(next) = response.data().await {
                        let chunk = next.unwrap();
                        match String::from_utf8(chunk.to_vec()) {
                            Ok(str) => {
                                res.push_str(&str);
                            },
                            Err(e) => {
                                error!("unable to convert chunk to UTF-8: {:?}", e);
                            }
                        }
                    }
                    return Some(res);
                },
                Err(e) => {
                    error!("could not get response: {:}", e);
                    return None;
                }
            }
        },
        Some("https") => {
            let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
            // println!("client: {:?}, url: {}", client, url);
            match client.get(url).await {
                Ok(mut response) => {
                    trace!("Response: {}, Headers {:#?}",
                           response.status(), response.headers());
                    let mut res = String::new();
                    if response.status() == 200 {
                        let jsonp = response.headers().contains_key(CONTENT_TYPE) &&
                            response.headers()[CONTENT_TYPE] == "application/json";
                        trace!("JSON? {}", jsonp);
                        while let Some(next) = response.data().await {
                            let chunk = next.unwrap();
                            match String::from_utf8(chunk.to_vec()) {
                                Ok(str) => {
                                    res.push_str(&str);
                                },
                                Err(e) => {
                                    error!("unable to convert chunk to UTF-8: {:?}", e);
                                }
                            }
                        }
                        if !jsonp {
                            error!("content not JSON: {}", res);
                            return None;
                        }
                    }
                    return Some(res);
                },
                Err(e) => {
                    error!("could not get response: {:}", e);
                    return None;
                }
            }
        },
        _ => {
            error!("unsupported scheme {:?}", url.scheme_str());
            return None;
        }
    };
}
/*
 */

/*
fn handle_response(response: Option<String>) {
    // println!("\nRESPONSE: {:?}", response);
    println!();
    match response {
        Some(res) => {
            println!("RESPONSE: {}", res);
        }
        None => {
            println!("RESPONSE: None");
        }
    }
}
 */

/// Interactive Mode
pub fn act<'b>(config: &'b mut Config<'b>) -> i32  {
    println!("interactive mode");
    let ttyp = is_a_tty(STDIN_FD) && ! config.is("disable_tty");
    if ttyp {
        stdin_raw_mode();
    } else {
        warn!("not a TTY, use ^D to quit");
    }
    let mut interactive = Interactive::new(config, ttyp);
    interactive.run().unwrap_or_else(|e| {
        error!("interactive error: {:?}", e);
    });
    if ttyp {
        restore_terminal(0);
    }
    0
}
