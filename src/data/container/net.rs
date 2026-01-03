use super::types::NetSample;

pub fn netns_id_for_pid(pid: u32) -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let path = format!("/proc/{pid}/ns/net");
        let target = std::fs::read_link(path).ok()?;
        parse_netns_target(&target.to_string_lossy())
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        None
    }
}

pub fn net_sample_for_pid(pid: u32) -> Option<NetSample> {
    #[cfg(target_os = "linux")]
    {
        let path = format!("/proc/{pid}/net/dev");
        let contents = std::fs::read_to_string(path).ok()?;
        parse_net_dev(&contents)
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        None
    }
}

fn parse_netns_target(value: &str) -> Option<u64> {
    let start = value.find('[')? + 1;
    let end = value[start..].find(']')? + start;
    value[start..end].parse::<u64>().ok()
}

fn parse_net_dev(contents: &str) -> Option<NetSample> {
    let mut sample = NetSample::default();
    let mut found = false;
    for line in contents.lines().skip(2) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(2, ':');
        let iface = parts.next().map(str::trim)?;
        let rest = parts.next()?.split_whitespace().collect::<Vec<_>>();
        if rest.len() < 9 {
            continue;
        }
        if iface.is_empty() {
            continue;
        }
        let Ok(rx_bytes) = rest[0].parse::<u64>() else {
            continue;
        };
        let Ok(tx_bytes) = rest[8].parse::<u64>() else {
            continue;
        };
        sample.rx_bytes = sample.rx_bytes.saturating_add(rx_bytes);
        sample.tx_bytes = sample.tx_bytes.saturating_add(tx_bytes);
        found = true;
    }
    if found { Some(sample) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_netns_target_reads_inode() {
        let input = "net:[4026531993]";
        assert_eq!(parse_netns_target(input), Some(4026531993));
    }

    #[test]
    fn parse_net_dev_sums_rx_tx() {
        let input = "\
Inter-|   Receive                                                |  Transmit\n\
 face |bytes packets errs drop fifo frame compressed multicast|bytes packets errs drop fifo colls carrier compressed\n\
  eth0: 1024 0 0 0 0 0 0 0 2048 0 0 0 0 0 0 0\n\
    lo: 512 0 0 0 0 0 0 0 1024 0 0 0 0 0 0 0\n";
        let sample = parse_net_dev(input).unwrap();
        assert_eq!(sample.rx_bytes, 1536);
        assert_eq!(sample.tx_bytes, 3072);
    }
}
