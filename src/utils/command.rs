use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

pub fn run_command_with_timeout(command: &str, args: &[&str], timeout: Duration) -> Option<String> {
    let mut child = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    let stdout = child.stdout.take()?;
    let stderr = child.stderr.take()?;
    let (out_tx, out_rx) = mpsc::channel();
    let (err_tx, err_rx) = mpsc::channel();

    thread::spawn(move || {
        let mut reader = io::BufReader::new(stdout);
        let mut output = String::new();
        let _ = reader.read_to_string(&mut output);
        let _ = out_tx.send(output);
    });

    thread::spawn(move || {
        let mut reader = io::BufReader::new(stderr);
        let mut output = String::new();
        let _ = reader.read_to_string(&mut output);
        let _ = err_tx.send(output);
    });

    let start = Instant::now();
    let mut timed_out = false;
    let success = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                break status.success();
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    timed_out = true;
                    let _ = child.kill();
                    let _ = child.wait();
                    break false;
                }
            }
            Err(_) => return None,
        }
        thread::sleep(Duration::from_millis(10));
    };

    let output = out_rx.recv().ok()?;
    let error_output = err_rx.recv().unwrap_or_default();
    if success {
        Some(output)
    } else {
        let command_display = if args.is_empty() {
            command.to_string()
        } else {
            format!("{command} {}", args.join(" "))
        };
        let stderr = error_output.trim();
        if !stderr.is_empty() {
            eprintln!("Command `{command_display}` failed: {stderr}");
        } else if timed_out {
            eprintln!("Command `{command_display}` timed out after {timeout:?}");
        } else {
            eprintln!("Command `{command_display}` failed");
        }
        None
    }
}
