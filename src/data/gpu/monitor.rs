use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::{GpuSnapshot, probe_gpus};

pub fn start_gpu_monitor() -> mpsc::Receiver<GpuSnapshot> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let interval = Duration::from_secs(2);
        loop {
            let snapshot = probe_gpus();
            if tx.send(snapshot).is_err() {
                break;
            }
            thread::sleep(interval);
        }
    });
    rx
}
