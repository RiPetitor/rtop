use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::{DrmProcessTracker, GpuSnapshot, probe_gpus_with_tracker};

pub fn start_gpu_monitor(interval: Duration) -> mpsc::Receiver<GpuSnapshot> {
    let (tx, rx) = mpsc::channel();
    let interval = interval.max(Duration::from_millis(100));
    thread::spawn(move || {
        let mut drm_tracker = DrmProcessTracker::new();
        loop {
            let snapshot = probe_gpus_with_tracker(&mut drm_tracker);
            if tx.send(snapshot).is_err() {
                break;
            }
            thread::sleep(interval);
        }
    });
    rx
}
