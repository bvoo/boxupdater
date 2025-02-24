// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime};
use tauri::Window;
use futures_util::StreamExt;
use reqwest::Client;
use tauri::Emitter;
use std::sync::Mutex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

// Cache structures
struct CacheEntry<T> {
    data: T,
    expires_at: SystemTime,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: SystemTime::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
}

static RELEASES_CACHE: Lazy<Mutex<HashMap<String, CacheEntry<Vec<Release>>>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));
static REPOSITORIES_CACHE: Lazy<Mutex<Option<CacheEntry<Vec<Repository>>>>> = 
    Lazy::new(|| Mutex::new(None));

#[derive(Serialize, Deserialize, Clone)]
struct Repository {
    name: String,
    owner: String,
    description: String,
    asset_filter: String,
}

#[derive(Serialize, Deserialize)]
struct Config {
    repositories: Vec<Repository>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Release {
    name: String,
    tag_name: String,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Clone, Serialize)]
struct DownloadProgress {
    progress: u32,
}

#[tauri::command]
async fn get_repositories() -> Result<Vec<Repository>, String> {
    const CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes cache
    
    // Check cache first
    {
        let cache = REPOSITORIES_CACHE.lock().unwrap();
        if let Some(entry) = cache.as_ref() {
            if !entry.is_expired() {
                return Ok(entry.data.clone());
            }
        }
    }
    
    // Cache miss or expired, fetch fresh data
    let config_str = include_str!("./repositories.json");
    let config: Config =
        serde_json::from_str(config_str).map_err(|e| format!("Failed to parse config: {}", e))?;
    
    // Update cache
    let repositories = config.repositories.clone();
    {
        let mut cache = REPOSITORIES_CACHE.lock().unwrap();
        *cache = Some(CacheEntry::new(repositories.clone(), CACHE_TTL));
    }
    
    Ok(repositories)
}

#[tauri::command]
async fn get_releases(repo_name: String) -> Result<Vec<Release>, String> {
    const CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes cache
    
    // Check cache first
    {
        let cache = RELEASES_CACHE.lock().unwrap();
        if let Some(entry) = cache.get(&repo_name) {
            if !entry.is_expired() {
                return Ok(entry.data.clone());
            }
        }
    }

    // Cache miss or expired, fetch fresh data
    let config_str = include_str!("./repositories.json");
    let config: Config =
        serde_json::from_str(config_str).map_err(|e| format!("Failed to parse config: {}", e))?;

    let repo = config
        .repositories
        .into_iter()
        .find(|r| r.name == repo_name)
        .ok_or_else(|| "Repository not found".to_string())?;

    let url = format!(
        "https://api.github.com/repos/{}/{}/releases",
        repo.owner, repo.name
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "boxupdater")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let releases: Vec<GithubRelease> = response.json().await.map_err(|e| e.to_string())?;

    let regex = regex::Regex::new(&repo.asset_filter)
        .map_err(|e| format!("Invalid asset filter regex: {}", e))?;

    let results: Vec<Release> = releases
        .into_iter()
        .flat_map(|release| {
            release
                .assets
                .into_iter()
                .filter(|asset| regex.is_match(&asset.name))
                .map(move |asset| Release {
                    name: asset.name,
                    tag_name: release.tag_name.clone(),
                    browser_download_url: asset.browser_download_url,
                })
        })
        .collect();

    // Update cache
    {
        let mut cache = RELEASES_CACHE.lock().unwrap();
        cache.insert(repo_name, CacheEntry::new(results.clone(), CACHE_TTL));
    }

    Ok(results)
}

#[tauri::command]
async fn write_to_rp2(is_nuke: bool, file_data: Option<Vec<u8>>) -> Result<(), String> {
    // Find RPI-RP2 drive
    let drives = find_rp2_drive().ok_or("RPI-RP2 drive not found")?;

    if is_nuke {
        // Write flash_nuke.uf2
        let nuke_data = include_bytes!("../../public/flash_nuke.uf2");
        fs::write(&drives.join("flash_nuke.uf2"), nuke_data)
            .map_err(|e| format!("Failed to write flash_nuke.uf2: {}", e))?;

        // Start checking if drive disappears
        let mut check_attempts = 0;
        while find_rp2_drive().is_some() {
            thread::sleep(Duration::from_millis(500));
            check_attempts += 1;
            if check_attempts > 20 { // 10 second timeout
                return Err("Device did not disconnect after flashing".to_string());
            }
        }

        // Wait for drive to reappear
        wait_for_drive_cycle()?;
    } else if let Some(firmware_data) = file_data {
        // Write the actual firmware
        fs::write(&drives.join("firmware.uf2"), firmware_data)
            .map_err(|e| format!("Failed to write firmware: {}", e))?;
        
        // Start checking if drive disappears
        let mut check_attempts = 0;
        while find_rp2_drive().is_some() {
            thread::sleep(Duration::from_millis(500));
            check_attempts += 1;
            if check_attempts > 20 { // 10 second timeout
                return Err("Device did not disconnect after flashing".to_string());
            }
        }
    }

    Ok(())
}

#[tauri::command]
async fn download_firmware(window: Window, url: String) -> Result<Vec<u8>, String> {
    let client = Client::new();
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut buffer = Vec::new();

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        buffer.extend_from_slice(&chunk);
        downloaded += chunk.len() as u64;
        
        if total_size > 0 {
            let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u32;
            let _ = window.emit_to("main", "download-progress", DownloadProgress { progress })
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(buffer)
}

#[tauri::command]
async fn check_rp2_drive() -> bool {
    find_rp2_drive().is_some()
}

fn find_rp2_drive() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        for drive_letter in b'A'..=b'Z' {
            let path = PathBuf::from(format!("{}:\\", drive_letter as char));
            if let Ok(metadata) = fs::metadata(&path) {
                if metadata.is_dir() {
                    let volume_name = winapi_volume_name(&path);
                    if volume_name.as_deref() == Some("RPI-RP2") {
                        return Some(path);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let volumes = PathBuf::from("/Volumes");
        if let Ok(entries) = fs::read_dir(volumes) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    if path.file_name().map(|n| n.to_string_lossy()) == Some("RPI-RP2".into()) {
                        return Some(path);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        
        // Check common mount points
        if let Ok(user) = std::env::var("USER") {
            let mount_points = vec![
                format!("/media/{}/RPI-RP2", user),
                format!("/run/media/{}/RPI-RP2", user),
                "/mnt/RPI-RP2".to_string(),
            ];
            
            // First try direct path checks
            for mount_point in mount_points {
                let path = PathBuf::from(&mount_point);
                if path.exists() && path.is_dir() {
                    if path.join("INFO_UF2.TXT").exists() {
                        return Some(path);
                    }
                }
            }
        }

        // If direct checks fail, try using lsblk to find FAT filesystems
        if let Ok(output) = Command::new("lsblk")
            .args(["-o", "NAME,FSTYPE,MOUNTPOINT", "-n", "-J"])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
                    if let Some(devices) = json["blockdevices"].as_array() {
                        for device in devices {
                            if device["fstype"].as_str() == Some("vfat") {
                                if let Some(mount) = device["mountpoint"].as_str() {
                                    let path = PathBuf::from(mount);
                                    if path.join("INFO_UF2.TXT").exists() {
                                        return Some(path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

#[cfg(windows)]
fn winapi_volume_name(path: &PathBuf) -> Option<String> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::fileapi::GetVolumeInformationW;

    let wide_path: Vec<u16> = OsStr::new(path.as_os_str())
        .encode_wide()
        .chain(once(0))
        .collect();

    let mut volume_name: Vec<u16> = vec![0; 256];
    let mut volume_serial_number: DWORD = 0;
    let mut maximum_component_length: DWORD = 0;
    let mut file_system_flags: DWORD = 0;
    let mut file_system_name_buffer: Vec<u16> = vec![0; 256];

    let success = unsafe {
        GetVolumeInformationW(
            wide_path.as_ptr(),
            volume_name.as_mut_ptr(),
            volume_name.len() as DWORD,
            &mut volume_serial_number,
            &mut maximum_component_length,
            &mut file_system_flags,
            file_system_name_buffer.as_mut_ptr(),
            file_system_name_buffer.len() as DWORD,
        ) != 0
    };

    if success {
        let len = volume_name
            .iter()
            .position(|&x| x == 0)
            .unwrap_or(volume_name.len());
        String::from_utf16_lossy(&volume_name[..len]).into()
    } else {
        None
    }
}

fn wait_for_drive_cycle() -> Result<(), String> {
    // Wait for drive to disappear
    while find_rp2_drive().is_some() {
        thread::sleep(Duration::from_millis(100));
    }

    // Wait for drive to reappear
    let mut attempts = 0;
    while find_rp2_drive().is_none() {
        thread::sleep(Duration::from_millis(100));
        attempts += 1;
        if attempts > 100 {
            // 10 second timeout
            return Err("Timeout waiting for device to reconnect".to_string());
        }
    }

    // Give the drive a moment to stabilize
    thread::sleep(Duration::from_millis(500));
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            write_to_rp2,
            download_firmware,
            get_repositories,
            get_releases,
            check_rp2_drive
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
