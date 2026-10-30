use super::*;
use std::net::{Ipv6Addr, SocketAddrV6};

pub fn ensure_port_free(port: u16) -> Result<()> {
    let owners = listener_details(port);
    if !owners.is_empty() {
        return Err(conflict_message(port, &owners));
    }
    // Probe wildcard addresses in both families, sequentially. Holding the IPv4
    // socket while binding a dual-stack IPv6 socket would conflict with ourselves.
    for address in [
        std::net::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)),
        std::net::SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0)),
    ] {
        match TcpListener::bind(address) {
            Ok(listener) => drop(listener),
            Err(error) if address.is_ipv6() && ipv6_unavailable(&error) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => {
                return Err(conflict_message(port, &listener_details(port)));
            }
            Err(error) => return Err(format!("无法检查端口 {port}（{address}）：{error}")),
        }
    }
    Ok(())
}
fn ipv6_unavailable(error: &std::io::Error) -> bool {
    if matches!(
        error.kind(),
        std::io::ErrorKind::Unsupported | std::io::ErrorKind::AddrNotAvailable
    ) {
        return true;
    }
    #[cfg(unix)]
    if matches!(
        error.raw_os_error(),
        Some(libc::EAFNOSUPPORT | libc::EPROTONOSUPPORT)
    ) {
        return true;
    }
    false
}
fn conflict_message(port: u16, owners: &[String]) -> String {
    let detail = if owners.is_empty() {
        String::new()
    } else {
        format!("：{}", owners.join("；"))
    };
    format!("端口 {port} 已被占用{detail}。请先停止占用进程，或修改此服务的端口后再启动。")
}
#[cfg(target_os = "macos")]
fn listener_details(port: u16) -> Vec<String> {
    let Ok(out) = Command::new("/usr/sbin/lsof")
        .args(["-nP", &format!("-iTCP:{port}"), "-sTCP:LISTEN", "-Fpcnt"])
        .output()
    else {
        return Vec::new();
    };
    parse_listeners(&String::from_utf8_lossy(&out.stdout))
}
#[cfg(not(target_os = "macos"))]
fn listener_details(_port: u16) -> Vec<String> {
    Vec::new()
}

fn parse_listeners(text: &str) -> Vec<String> {
    let (mut pid, mut name, mut family) = (String::new(), String::new(), String::new());
    let mut result = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let Some(value) = line.get(1..) else {
            continue;
        };
        let value: String = value
            .chars()
            .filter(|c| !c.is_control())
            .take(120)
            .collect();
        match line.as_bytes()[0] {
            b'p' => {
                pid = value;
                name.clear();
                family.clear();
            }
            b'c' => name = value,
            b'f' => family.clear(),
            b't' => family = value,
            b'n' if !pid.is_empty() && pid.chars().all(|c| c.is_ascii_digit()) => {
                let item = format!(
                    "{}（PID {}，{} {}）",
                    if name.is_empty() { "进程" } else { &name },
                    pid,
                    family,
                    value
                );
                if !result.contains(&item) {
                    result.push(item);
                }
            }
            _ => {}
        }
    }
    result.truncate(4);
    result
}

pub(super) fn exit_error(
    port: u16,
    exit: std::process::ExitStatus,
    ready: bool,
    log_path: &Path,
    offset: u64,
) -> String {
    if !ready {
        // Only inspect output from this attempt, not an old failed launch.
        let collision = (|| -> std::io::Result<bool> {
            let mut file = File::open(log_path)?;
            let end = file.metadata()?.len();
            let start = if end < offset { 0 } else { offset };
            file.seek(SeekFrom::Start(start.max(end.saturating_sub(8192))))?;
            let mut output = String::new();
            file.read_to_string(&mut output)?;
            let lower = output.to_ascii_lowercase();
            Ok(output.contains("EADDRINUSE") || lower.contains("address already in use"))
        })()
        .unwrap_or(false);
        let owners = listener_details(port);
        if collision || !owners.is_empty() {
            return format!("启动失败（{exit}）：{}", conflict_message(port, &owners));
        }
    }
    format!("服务意外退出（{exit}）。请查看运行日志中的具体错误。")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn listener_output_keeps_ipv4_and_ipv6_process_details() {
        let owners = parse_listeners(
            "p123\ncnode\nf12\ntIPv6\nn*:3000\np456\ncpython\nf8\ntIPv4\nn127.0.0.1:3000\n",
        );
        assert_eq!(owners.len(), 2);
        assert!(owners[0].contains("node（PID 123，IPv6 *:3000）"));
        assert!(owners[1].contains("python（PID 456，IPv4 127.0.0.1:3000）"));
        assert!(parse_listeners("garbage\npnot-a-pid\nn*:3000").is_empty());
    }
}
