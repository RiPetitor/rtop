use super::types::{ContainerKey, ContainerRuntime};

pub fn container_key_for_pid(pid: u32) -> Option<ContainerKey> {
    #[cfg(target_os = "linux")]
    {
        let path = format!("/proc/{pid}/cgroup");
        let contents = std::fs::read_to_string(path).ok()?;
        parse_cgroup(&contents)
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        None
    }
}

fn parse_cgroup(contents: &str) -> Option<ContainerKey> {
    for line in contents.lines() {
        if let Some(path) = line.splitn(3, ':').nth(2)
            && let Some(key) = parse_cgroup_path(path.trim())
        {
            return Some(key);
        }
    }
    None
}

fn parse_cgroup_path(path: &str) -> Option<ContainerKey> {
    if path.is_empty() {
        return None;
    }

    let runtime = detect_runtime(path)?;
    let segments = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    if let Some(id) = extract_runtime_id(&segments) {
        return Some(ContainerKey { runtime, id });
    }

    if let Some(id) = segments.iter().find_map(|segment| hex_segment(segment)) {
        return Some(ContainerKey { runtime, id });
    }

    if runtime == ContainerRuntime::Kubernetes
        && let Some(id) = segments.iter().find_map(|segment| pod_segment(segment))
    {
        return Some(ContainerKey { runtime, id });
    }

    None
}

fn detect_runtime(path: &str) -> Option<ContainerRuntime> {
    if path.contains("kubepods") {
        return Some(ContainerRuntime::Kubernetes);
    }
    if path.contains("libpod") || path.contains("podman") {
        return Some(ContainerRuntime::Podman);
    }
    if path.contains("docker") {
        return Some(ContainerRuntime::Docker);
    }
    if path.contains("crio") {
        return Some(ContainerRuntime::Crio);
    }
    if path.contains("containerd") {
        return Some(ContainerRuntime::Containerd);
    }
    None
}

fn extract_runtime_id(segments: &[&str]) -> Option<String> {
    for (idx, segment) in segments.iter().enumerate() {
        if let Some(id) = strip_scope_prefix(segment, "docker-") {
            return Some(id);
        }
        if let Some(id) = strip_scope_prefix(segment, "libpod-") {
            return Some(id);
        }
        if let Some(id) = strip_scope_prefix(segment, "podman-") {
            return Some(id);
        }
        if let Some(id) = strip_scope_prefix(segment, "crio-") {
            return Some(id);
        }
        if let Some(id) = strip_scope_prefix(segment, "cri-containerd-") {
            return Some(id);
        }
        if let Some(id) = strip_scope_prefix(segment, "containerd-") {
            return Some(id);
        }

        if let Some(id) = next_segment_id(segments, idx, "docker") {
            return Some(id);
        }
        if let Some(id) = next_segment_id(segments, idx, "libpod") {
            return Some(id);
        }
        if let Some(id) = next_segment_id(segments, idx, "podman") {
            return Some(id);
        }
        if let Some(id) = next_segment_id(segments, idx, "crio") {
            return Some(id);
        }
        if let Some(id) = next_segment_id(segments, idx, "containerd") {
            return Some(id);
        }
    }
    None
}

fn next_segment_id(segments: &[&str], idx: usize, marker: &str) -> Option<String> {
    if segments.get(idx)? != &marker {
        return None;
    }
    let next = segments.get(idx + 1)?;
    let next = trim_suffixes(next);
    if next.is_empty() {
        return None;
    }
    Some(next.to_string())
}

fn strip_scope_prefix(segment: &str, prefix: &str) -> Option<String> {
    let rest = segment.strip_prefix(prefix)?;
    let rest = trim_suffixes(rest);
    if rest.is_empty() {
        return None;
    }
    Some(rest.to_string())
}

fn trim_suffixes(value: &str) -> &str {
    let trimmed = value
        .strip_suffix(".scope")
        .or_else(|| value.strip_suffix(".slice"))
        .or_else(|| value.strip_suffix(".service"));
    trimmed.unwrap_or(value)
}

fn hex_segment(segment: &str) -> Option<String> {
    let trimmed = trim_suffixes(segment);
    if trimmed.len() < 8 {
        return None;
    }
    if trimmed.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Some(trimmed.to_string());
    }
    None
}

fn pod_segment(segment: &str) -> Option<String> {
    let trimmed = trim_suffixes(segment);
    let idx = trimmed.find("pod")?;
    let rest = trimmed[idx + 3..].trim_start_matches('_');
    if rest.is_empty() {
        return None;
    }
    Some(format!("pod{rest}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_docker_scope() {
        let input = "0::/user.slice/user-1000.slice/user@1000.service/app.slice/docker-0123456789abcdef.scope";
        let key = parse_cgroup(input).unwrap();
        assert_eq!(key.runtime, ContainerRuntime::Docker);
        assert_eq!(key.id, "0123456789abcdef");
    }

    #[test]
    fn parse_docker_legacy() {
        let input = "1:name=systemd:/docker/0123456789abcdef";
        let key = parse_cgroup(input).unwrap();
        assert_eq!(key.runtime, ContainerRuntime::Docker);
        assert_eq!(key.id, "0123456789abcdef");
    }

    #[test]
    fn parse_libpod_scope() {
        let input = "0::/user.slice/libpod-aaaaaaaaaaaaaaaa.scope";
        let key = parse_cgroup(input).unwrap();
        assert_eq!(key.runtime, ContainerRuntime::Podman);
        assert_eq!(key.id, "aaaaaaaaaaaaaaaa");
    }

    #[test]
    fn parse_kube_containerd_scope() {
        let input = "0::/kubepods.slice/kubepods-besteffort.slice/kubepods-besteffort-pod123.slice/cri-containerd-bbbbbbbbbbbbbbbb.scope";
        let key = parse_cgroup(input).unwrap();
        assert_eq!(key.runtime, ContainerRuntime::Kubernetes);
        assert_eq!(key.id, "bbbbbbbbbbbbbbbb");
    }

    #[test]
    fn parse_kube_crio_scope() {
        let input = "0::/kubepods.slice/kubepods-pod123.slice/crio-cccccccccccccccc.scope";
        let key = parse_cgroup(input).unwrap();
        assert_eq!(key.runtime, ContainerRuntime::Kubernetes);
        assert_eq!(key.id, "cccccccccccccccc");
    }
}
