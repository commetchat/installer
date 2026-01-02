use std::{
    fs::DirEntry,
    io::{self, Cursor},
    path::Path,
};

use log::info;
use octocrab::Error;
use pgp::composed::{Deserializable, DetachedSignature, SignedPublicKey};
use platform_dirs::AppDirs;
use shortcuts_rs::ShellLink;
use sysinfo::{get_current_pid, System};
use zip::{read::root_dir_common_filter, result::ZipError};

use crate::config;

pub struct ReleaseInfo {
    pub name: String,
    pub binary_url: String,
    pub signature_url: String,
}

pub async fn fetch_info(
    owner: &str,
    repo: &str,
    asset_name: &str,
    signature_name: &str,
    use_prereleases: bool,
) -> Result<Option<ReleaseInfo>, Error> {
    _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let api = octocrab::instance();

    let releases = api
        .repos(owner, repo)
        .releases()
        .list()
        .per_page(100)
        .send()
        .await?;
    for release in releases.items.iter() {
        if use_prereleases == false && release.prerelease {
            continue;
        }

        info!("Found Release: {}", release.tag_name);

        let mut binary_url: Option<String> = None;
        let mut signature_url: Option<String> = None;

        for asset in release.assets.iter() {
            if asset.name == asset_name.to_string() {
                info!("Has Asset: {}", asset.name);
                binary_url = Some(asset.browser_download_url.to_string());
            }

            if asset.name == signature_name.to_string() {
                info!("Has Signature: {}", asset.name);
                signature_url = Some(asset.browser_download_url.to_string());
            }
        }

        if let Some(binary_url) = binary_url {
            if let Some(signature_url) = signature_url {
                return Ok(Some(ReleaseInfo {
                    name: release.tag_name.clone(),
                    binary_url: binary_url,
                    signature_url: signature_url,
                }));
            }
        }
    }

    Ok(None)
}

pub struct ReleaseDownload {
    pub binary: Vec<u8>,
    pub signature: Vec<u8>,
}

pub async fn download_release<F>(
    info: ReleaseInfo,
    set_text: &F,
) -> Result<ReleaseDownload, reqwest::Error>
where
    F: Fn(&str) -> (),
{
    let client = reqwest::Client::new();
    let mut resp = client.get(info.binary_url).send().await?;

    let total_bytes = resp.content_length().unwrap_or_default();

    let mut result = Vec::<u8>::new();

    while let Some(chunk) = resp.chunk().await? {
        let mut data = chunk.to_vec().clone();

        result.append(&mut data);
        let a = result.len() as f32;
        let b = total_bytes as f32;
        let progress = (a / b) * 100.0;

        set_text(&format!(
            "Downloading {}: {:0}%",
            info.name,
            progress.round()
        ));
    }

    set_text("Downloading signature");

    let client = reqwest::Client::new();
    let resp = client.get(info.signature_url).send().await?;

    let signature_bytes = resp.bytes().await;
    let signature_bytes = match signature_bytes {
        Ok(bytes) => bytes,
        Err(err) => {
            set_text("Failed to download signature");
            return Err(err);
        }
    };

    Ok(ReleaseDownload {
        binary: result,
        signature: signature_bytes.to_vec(),
    })
}

pub fn verify_release(download: &ReleaseDownload) -> bool {
    let public_key = SignedPublicKey::from_armor_single(Cursor::new(config::PUBLIC_KEY));

    let (public_key, _headers) = match public_key {
        Ok(key) => key,
        Err(_) => {
            return false;
        }
    };

    let (signature, _headers) =
        match DetachedSignature::from_armor_single(Cursor::new(download.signature.clone())) {
            Ok(sig) => sig,
            Err(_) => return false,
        };

    match signature.verify(&public_key, &download.binary) {
        Ok(_) => {
            log::info!("Signature is valid, continuing");
            return true;
        }
        Err(err) => {
            log::error!("could not validate signature: {:?}", err);
            return false;
        }
    }
}

pub struct ExtractionResult {
    pub directory: String,
}

pub async fn close_existing_sessions() {
    let result = AppDirs::new(Some(config::APP_ID), false).unwrap();
    let dir = result.data_dir.to_str().unwrap();

    let s = System::new_all();
    let processes = s.processes();
    let current_pid = get_current_pid().unwrap();

    for (pid, process) in processes.iter() {
        let exe = process.exe();
        let exe = match exe {
            Some(exe) => exe,
            None => continue,
        };

        // dont kill ourself
        if pid == &current_pid {
            continue;
        }

        if exe.starts_with(dir) {
            info!(
                "Killing process: [{}] Name: {}",
                pid,
                process.name().to_str().unwrap()
            );

            match process.kill_and_wait() {
                Ok(result) => {
                    log::info!("result: {:?}", result);
                }
                Err(err) => {
                    log::error!("Failed to kill process: {:?}", err)
                }
            }
        };
    }
}

pub async fn extract_release<F>(
    download: ReleaseDownload,
    set_text: &F,
) -> Result<ExtractionResult, ZipError>
where
    F: Fn(&str) -> (),
{
    let result = AppDirs::new(Some(config::APP_ID), false).unwrap();
    let dir = result.data_dir.to_str().unwrap();

    let temp = dir.to_string() + "_tmp";

    info!("extracting to: {}", temp);

    match std::fs::exists(&temp) {
        Ok(exists) => {
            if exists {
                match std::fs::remove_dir_all(&temp) {
                    Ok(_) => (),
                    Err(err) => {
                        set_text("Failed to clear temp directory");
                        return Err(ZipError::Io(err));
                    }
                }
            }
        }
        Err(err) => {
            set_text("Failed to check if temp dir already exists");
            return Err(ZipError::Io(err));
        }
    }

    match std::fs::exists(&temp) {
        Ok(exists) => {
            if !exists {
                match std::fs::create_dir(&temp) {
                    Ok(_) => (),
                    Err(err) => {
                        set_text("Failed to create temp directory");
                        return Err(ZipError::Io(err));
                    }
                }
            }
        }
        Err(err) => return Err(ZipError::Io(err)),
    };

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(download.binary))?;

    archive.extract_unwrapped_root_dir(&temp, root_dir_common_filter)?;

    info!("Extracted to temp dir, removing old installation");

    set_text("Removing old install");

    match remove_existing_install(dir).await {
        Ok(_) => (),
        Err(err) => {
            set_text("Failed to remove existing installation");
            log::error!("Error: {:?}", err);
            return Err(ZipError::Io(err));
        }
    }

    info!("Removed previous, moving temp install");

    match move_from_temp(&temp, dir).await {
        Ok(_) => (),
        Err(err) => {
            set_text("Failed to move install to correct location");
            return Err(ZipError::Io(err));
        }
    }

    info!("Done!");

    Ok(ExtractionResult {
        directory: dir.to_string(),
    })
}

async fn move_from_temp(temp: &str, dir: &str) -> Result<(), io::Error> {
    let entries = std::fs::read_dir(temp)?;

    let entries: Vec<DirEntry> = entries.map(|res| res.unwrap()).collect();
    for entry in entries.iter() {
        info!("Moving\t: {}", entry.path().to_str().unwrap());

        let name = entry.file_name();

        let new_name = Path::new(dir).join(name);

        info!("To\t\t: {}", new_name.to_str().unwrap());

        std::fs::rename(entry.path(), new_name)?;
    }

    std::fs::remove_dir(temp)?;
    Ok(())
}

async fn remove_existing_install(dir: &str) -> Result<(), io::Error> {
    if std::fs::exists(dir)? == false {
        return Ok(());
    }

    let entries = std::fs::read_dir(dir)?;

    let entries: Vec<DirEntry> = entries.map(|res| res.unwrap()).collect();

    let current_exe = std::env::current_exe().unwrap();

    for entry in entries.iter() {
        info!("Removing: {}", entry.path().to_str().unwrap());
        if entry.path() == current_exe || entry.file_name() == "installer" {
            info!("Skipping removing ourself");
            continue;
        }

        match entry.file_type() {
            Ok(file_type) => {
                if file_type.is_dir() {
                    std::fs::remove_dir_all(entry.path())?;
                }

                if file_type.is_file() {
                    std::fs::remove_file(entry.path())?;
                }
            }
            Err(_) => todo!(),
        }
    }

    Ok(())
}

pub async fn add_self_to_install_dir(result: &ExtractionResult) -> Result<(), io::Error> {
    let current_exe = std::env::current_exe().unwrap();

    let data = std::fs::read(&current_exe)?;

    let dir = result.directory.to_string();
    let new_path = Path::new(&dir);
    let new_path = new_path.join("installer");

    let _ = std::fs::create_dir_all(&new_path);

    let new_path = new_path.join("commet-installer.exe");

    if current_exe != new_path {
        std::fs::write(new_path, data)?;
    }

    Ok(())
}

pub async fn add_to_start_menu(result: &ExtractionResult) {
    let dir = AppDirs::new(None, false).unwrap();
    let dir = dir.config_dir;

    let start_menu_dir = Path::new(&dir);
    let shortcut = start_menu_dir
        .join("Microsoft\\Windows\\Start Menu\\Programs")
        .join(config::WINDOWS_SHORTCUT_NAME);

    let executable = Path::new(&result.directory);
    let executable = executable.join(config::WINDOWS_EXE_NAME);

    info!("Creating shortcut: {:?}", shortcut.to_str());

    let link = ShellLink::new(executable, None, None, None).unwrap();
    let result = link.create_lnk(shortcut);

    match result {
        Ok(_) => {
            info!("Created shortcut")
        }
        Err(err) => {
            log::error!("Failed to create shortcut: {:?}", err)
        }
    }

    info!("AppData: {}", dir.to_str().unwrap());
}

pub fn launch_app(result: &ExtractionResult) -> Result<(), io::Error> {
    let exe = Path::new(&result.directory);
    let exe = exe.join(config::WINDOWS_EXE_NAME);

    let exe = exe.to_str().unwrap().to_string();

    info!("Launching exe: {}", exe);

    let mut cmd = std::process::Command::new(exe);
    let _ = cmd.spawn();

    Ok(())
}
