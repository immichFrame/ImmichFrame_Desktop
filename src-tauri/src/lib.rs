use dirs::{config_dir, home_dir};
use serde::Serialize;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};

fn settings_dir() -> Result<std::path::PathBuf, String> {
    let save_dir = match std::env::consts::OS {
        "windows" => home_dir()
            .ok_or("Failed to get home directory")?
            .join("AppData")
            .join("Roaming")
            .join("immichFrame"),
        "macos" => home_dir()
            .ok_or("Failed to get home directory")?
            .join("Library")
            .join("Application Support")
            .join("immichFrame"),
        "linux" => config_dir()
            .ok_or("Failed to get config directory")?
            .join("immichFrame"),
        _ => home_dir().ok_or("Failed to get home directory")?,
    };

    if !save_dir.exists() {
        create_dir_all(&save_dir).map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    Ok(save_dir)
}

#[tauri::command]
fn save_url_to_file(url: String) -> Result<(), String> {
    let file_path = settings_dir()?.join("Settings.txt");
    let mut file = File::create(file_path).map_err(|e| format!("Failed to open file: {e}"))?;
    file.write_all(url.as_bytes())
        .map_err(|e| format!("Failed to write to file: {e}"))
}

#[tauri::command]
fn read_url_from_file() -> Result<String, String> {
    let file_path = settings_dir()?.join("Settings.txt");
    let mut file = File::open(file_path).map_err(|e| format!("Failed to open file: {e}"))?;

    let mut url = String::new();
    file.read_to_string(&mut url)
        .map_err(|e| format!("Failed to read file: {e}"))?;
    Ok(url)
}

#[tauri::command]
fn exit_app() {
    std::process::exit(0);
}

#[tauri::command]
fn restart_app() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("Failed to locate current exe: {e}"))?;
    std::process::Command::new(exe)
        .spawn()
        .map_err(|e| format!("Failed to restart app: {e}"))?;
    std::process::exit(0);
}

#[tauri::command]
fn read_immichframe_env(suffix: String) -> Option<String> {
    // Only allow reading env vars of the form IMMICHFRAME_<A-Z0-9_+>.
    if suffix.is_empty() {
        return None;
    }
    if !suffix
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
    {
        return None;
    }

    let key = format!("IMMICHFRAME_{suffix}");
    std::env::var(key).ok()
}

#[derive(Debug, Clone, Serialize)]
struct LinuxResourceStats {
    mem_total_kb: Option<u64>,
    mem_free_kb: Option<u64>,
    mem_available_kb: Option<u64>,
    swap_total_kb: Option<u64>,
    swap_free_kb: Option<u64>,
    cma_total_kb: Option<u64>,
    cma_free_kb: Option<u64>,
    self_rss_kb: Option<u64>,
    self_vmsize_kb: Option<u64>,
    webkit_rss_kb: Option<u64>,
    webkit_process_count: Option<u32>,
}

#[cfg(target_os = "linux")]
fn parse_meminfo_kb(contents: &str, key: &str) -> Option<u64> {
    // Lines look like: "MemAvailable:   123456 kB"
    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix(key) {
            let rest = rest.trim_start();
            let rest = rest.strip_prefix(':')?.trim_start();
            let number_str = rest.split_whitespace().next()?;
            return number_str.parse::<u64>().ok();
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn parse_status_kb(contents: &str, key: &str) -> Option<u64> {
    // Lines look like: "VmRSS:\t  12345 kB"
    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix(key) {
            let rest = rest.trim_start();
            let rest = rest.strip_prefix(':')?.trim_start();
            let number_str = rest.split_whitespace().next()?;
            return number_str.parse::<u64>().ok();
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn parse_ps_table(output: &str) -> Vec<(u32, u32, u64, String)> {
    // Expected columns: pid ppid rss_kb comm
    // rss from ps is in KiB.
    let mut rows = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(pid) = parts.next().and_then(|s| s.parse::<u32>().ok()) else {
            continue;
        };
        let Some(ppid) = parts.next().and_then(|s| s.parse::<u32>().ok()) else {
            continue;
        };
        let Some(rss_kb) = parts.next().and_then(|s| s.parse::<u64>().ok()) else {
            continue;
        };
        let comm = parts.next().unwrap_or("").to_string();
        rows.push((pid, ppid, rss_kb, comm));
    }
    rows
}

#[cfg(target_os = "linux")]
fn get_webkit_descendant_rss_kb() -> Option<(u64, u32)> {
    let self_pid = std::process::id();

    let output = std::process::Command::new("ps")
        .args(["-e", "-o", "pid=,ppid=,rss=,comm="])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
        .or_else(|| {
            // Fallback for ps variants that don't support '=' suppression.
            std::process::Command::new("ps")
                .args(["-e", "-o", "pid,ppid,rss,comm"])
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        String::from_utf8(o.stdout).ok()
                    } else {
                        None
                    }
                })
        })?;

    let mut rows = parse_ps_table(&output);
    if rows.is_empty() {
        // If the fallback output included a header, drop the first line and retry.
        if let Some((_first, rest)) = output.split_once('\n') {
            rows = parse_ps_table(rest);
        }
    }

    if rows.is_empty() {
        return None;
    }

    // Build pid -> (ppid, rss, comm)
    use std::collections::{HashMap, HashSet};
    let mut proc_map: HashMap<u32, (u32, u64, String)> = HashMap::new();
    for (pid, ppid, rss, comm) in rows {
        proc_map.insert(pid, (ppid, rss, comm));
    }

    // Find descendants of self.
    let mut descendants: HashSet<u32> = HashSet::new();
    let mut changed = true;
    while changed {
        changed = false;
        for (pid, (ppid, _rss, _comm)) in proc_map.iter() {
            if *ppid == self_pid || descendants.contains(ppid) {
                if descendants.insert(*pid) {
                    changed = true;
                }
            }
        }
    }

    let mut total_rss: u64 = 0;
    let mut count: u32 = 0;
    for pid in descendants {
        if let Some((_ppid, rss, comm)) = proc_map.get(&pid) {
            if comm.starts_with("WebKit") {
                total_rss = total_rss.saturating_add(*rss);
                count = count.saturating_add(1);
            }
        }
    }

    Some((total_rss, count))
}

#[tauri::command]
fn get_linux_resource_stats() -> Option<LinuxResourceStats> {
    #[cfg(target_os = "linux")]
    {
        let meminfo = std::fs::read_to_string("/proc/meminfo").ok();
        let status = std::fs::read_to_string("/proc/self/status").ok();

        let mem_total_kb = meminfo
            .as_deref()
            .and_then(|s| parse_meminfo_kb(s, "MemTotal"));
        let mem_free_kb = meminfo.as_deref().and_then(|s| parse_meminfo_kb(s, "MemFree"));
        let mem_available_kb = meminfo
            .as_deref()
            .and_then(|s| parse_meminfo_kb(s, "MemAvailable"));
        let swap_total_kb = meminfo
            .as_deref()
            .and_then(|s| parse_meminfo_kb(s, "SwapTotal"));
        let swap_free_kb = meminfo
            .as_deref()
            .and_then(|s| parse_meminfo_kb(s, "SwapFree"));
        let cma_total_kb = meminfo.as_deref().and_then(|s| parse_meminfo_kb(s, "CmaTotal"));
        let cma_free_kb = meminfo.as_deref().and_then(|s| parse_meminfo_kb(s, "CmaFree"));

        let self_rss_kb = status.as_deref().and_then(|s| parse_status_kb(s, "VmRSS"));
        let self_vmsize_kb = status.as_deref().and_then(|s| parse_status_kb(s, "VmSize"));

        let (webkit_rss_kb, webkit_process_count) = match get_webkit_descendant_rss_kb() {
            Some((rss, count)) => (Some(rss), Some(count)),
            None => (None, None),
        };

        Some(LinuxResourceStats {
            mem_total_kb,
            mem_free_kb,
            mem_available_kb,
            swap_total_kb,
            swap_free_kb,
            cma_total_kb,
            cma_free_kb,
            self_rss_kb,
            self_vmsize_kb,
            webkit_rss_kb,
            webkit_process_count,
        })
    }

    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            save_url_to_file,
            read_url_from_file,
            exit_app,
            restart_app,
            read_immichframe_env,
            get_linux_resource_stats
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
