//! CPU, memory, disk and network numbers for the island's status widgets. The front end asks every two
//! seconds while the island is hovered; the previous sample lives here so a call only needs the difference.
use serde::Serialize;
use std::sync::Mutex;
use std::time::Instant;
use windows::core::PCWSTR;
use windows::Win32::Foundation::FILETIME;
use windows::Win32::NetworkManagement::IpHelper::{FreeMibTable, GetIfTable2, MIB_IF_TABLE2};
use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows::Win32::System::Threading::GetSystemTimes;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SystemStats {
    pub cpu: u8,
    pub ram: u8,
    pub disk: u8,
    /// Bytes per second.
    pub net_down: u64,
    pub net_up: u64,
}

/// Idle, kernel (which includes idle) and user time, in 100 ns units.
type CpuTimes = (u64, u64, u64);

struct Prev {
    at: Instant,
    cpu: CpuTimes,
    net: (u64, u64),
}

static PREV: Mutex<Option<Prev>> = Mutex::new(None);

/// Busy share of the time between two samples, 0 to 100. Without a previous sample there is nothing to compare: 0.
fn cpu_percent(prev: Option<CpuTimes>, now: CpuTimes) -> u8 {
    let Some(prev) = prev else { return 0 };
    let total = now.1.saturating_sub(prev.1) + now.2.saturating_sub(prev.2);
    let idle = now.0.saturating_sub(prev.0);
    if total == 0 {
        return 0;
    }
    (100 * total.saturating_sub(idle) / total).min(100) as u8
}

/// Bytes per second between two counter readings. A counter that went backwards (adapter reset) or no
/// elapsed time gives 0.
fn rate(prev: u64, now: u64, elapsed_ms: u64) -> u64 {
    if elapsed_ms == 0 || now < prev {
        return 0;
    }
    (now - prev) * 1000 / elapsed_ms
}

fn ft(t: FILETIME) -> u64 {
    ((t.dwHighDateTime as u64) << 32) | t.dwLowDateTime as u64
}

fn cpu_times() -> Option<CpuTimes> {
    let (mut idle, mut kernel, mut user) = (FILETIME::default(), FILETIME::default(), FILETIME::default());
    unsafe { GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)).ok()? };
    Some((ft(idle), ft(kernel), ft(user)))
}

fn ram_percent() -> u8 {
    let mut m = MEMORYSTATUSEX { dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32, ..Default::default() };
    if unsafe { GlobalMemoryStatusEx(&mut m) }.is_ok() { m.dwMemoryLoad.min(100) as u8 } else { 0 }
}

fn disk_percent() -> u8 {
    let drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
    let root: Vec<u16> = format!("{drive}\\").encode_utf16().chain(std::iter::once(0)).collect();
    let (mut free, mut total) = (0u64, 0u64);
    let ok = unsafe { GetDiskFreeSpaceExW(PCWSTR(root.as_ptr()), None, Some(&mut total), Some(&mut free)) }.is_ok();
    if !ok || total == 0 {
        return 0;
    }
    (100 * total.saturating_sub(free) / total).min(100) as u8
}

/// Received and sent bytes summed over the interfaces that are up (loopback excluded).
fn net_counters() -> (u64, u64) {
    const IF_OPER_STATUS_UP: i32 = 1;
    const IF_TYPE_SOFTWARE_LOOPBACK: u32 = 24;
    let mut table: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
    if unsafe { GetIfTable2(&mut table) }.is_err() || table.is_null() {
        return (0, 0);
    }
    let (mut down, mut up) = (0u64, 0u64);
    unsafe {
        let rows = std::slice::from_raw_parts((*table).Table.as_ptr(), (*table).NumEntries as usize);
        for r in rows {
            if r.OperStatus.0 == IF_OPER_STATUS_UP && r.Type != IF_TYPE_SOFTWARE_LOOPBACK {
                down += r.InOctets;
                up += r.OutOctets;
            }
        }
        FreeMibTable(table as *const _);
    }
    (down, up)
}

#[tauri::command]
pub fn get_system_stats() -> SystemStats {
    let cpu_now = cpu_times().unwrap_or((0, 0, 0));
    let net_now = net_counters();
    let now = Instant::now();
    let mut guard = PREV.lock().unwrap_or_else(|e| e.into_inner());
    let (cpu, net_down, net_up) = match guard.as_ref() {
        Some(p) => {
            let ms = now.duration_since(p.at).as_millis() as u64;
            (cpu_percent(Some(p.cpu), cpu_now), rate(p.net.0, net_now.0, ms), rate(p.net.1, net_now.1, ms))
        }
        None => (0, 0, 0),
    };
    *guard = Some(Prev { at: now, cpu: cpu_now, net: net_now });
    SystemStats { cpu, ram: ram_percent(), disk: disk_percent(), net_down, net_up }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_is_zero_without_a_previous_sample() {
        assert_eq!(cpu_percent(None, (10, 20, 30)), 0);
    }

    #[test]
    fn cpu_percent_is_busy_over_total() {
        // total = kernel delta 100 + user delta 100 = 200, idle delta 50
        assert_eq!(cpu_percent(Some((100, 200, 100)), (150, 300, 200)), 75);
    }

    #[test]
    fn cpu_percent_stays_within_0_100() {
        assert_eq!(cpu_percent(Some((0, 0, 0)), (500, 100, 0)), 0);
        assert_eq!(cpu_percent(Some((5, 5, 5)), (5, 5, 5)), 0);
        assert!(cpu_percent(Some((0, 0, 0)), (0, 10, 10)) <= 100);
    }

    #[test]
    fn rate_is_bytes_per_second() {
        assert_eq!(rate(1000, 3000, 2000), 1000);
    }

    #[test]
    fn rate_is_zero_when_the_counter_goes_backwards() {
        assert_eq!(rate(5000, 100, 2000), 0);
    }

    #[test]
    fn rate_is_zero_for_zero_elapsed_time() {
        assert_eq!(rate(0, 100, 0), 0);
    }
}
