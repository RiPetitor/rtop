use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

pub fn run_command_with_timeout(command: &str, args: &[&str], timeout: Duration) -> Option<String> {
    let mut child = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let stdout = child.stdout.take()?;
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut reader = io::BufReader::new(stdout);
        let mut output = String::new();
        let _ = reader.read_to_string(&mut output);
        let _ = tx.send(output);
    });
    let start = Instant::now();
    let success = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                break status.success();
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    break false;
                }
            }
            Err(_) => return None,
        }
        thread::sleep(Duration::from_millis(10));
    };

    let output = rx.recv().ok()?;
    if success { Some(output) } else { None }
}
