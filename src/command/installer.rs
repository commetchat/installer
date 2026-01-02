use std::time::Duration;

use log::info;

use crate::{
    cli::Args,
    command::install_utils::{
        add_self_to_install_dir, add_to_start_menu, close_existing_sessions, download_release,
        extract_release, fetch_info, launch_app, verify_release,
    },
    config,
};

pub async fn install<F>(args: Args, set_text: F) -> bool
where
    F: Fn(&str) -> (),
{
    info!("Starting installation");

    set_text("Fetching releases...");

    let info = fetch_info(
        config::REPO_OWNER,
        config::REPO,
        config::WINDOWS_BINARY_NAME,
        config::WINDOWS_SIGNATURE_NAME,
        args.prerelease,
    )
    .await;

    let info = match info {
        Ok(info) => info,
        Err(_) => {
            set_text("Failed to fetch release info");
            return false;
        }
    };

    let info = match info {
        Some(info) => info,
        None => {
            set_text("Failed to fetch release info");
            return false;
        }
    };

    info!("Got binary url: {}", info.binary_url);
    info!("Got signature url: {}", info.signature_url);

    set_text(format!("Downloading {}", info.name).as_str());

    let download = download_release(info, &set_text).await;

    let download = match download {
        Ok(download) => download,
        Err(_) => {
            set_text("An error occurred while downloading");
            return false;
        }
    };

    set_text(format!("Download Finished").as_str());

    set_text("Verifying release");
    tokio::time::sleep(Duration::from_millis(1000)).await;

    if verify_release(&download) == false {
        set_text("Failed to verify release...");
        return false;
    }

    set_text("Verified!");
    tokio::time::sleep(Duration::from_millis(1000)).await;

    close_existing_sessions().await;

    set_text("Extracting...");
    let result = match extract_release(download, &set_text).await {
        Ok(result) => result,
        Err(_) => {
            return false;
        }
    };

    match add_self_to_install_dir(&result).await {
        Ok(_) => (),
        Err(_) => {
            set_text("Failed to add installer to app directory");
            return false;
        }
    }

    add_to_start_menu(&result).await;

    match launch_app(&result) {
        Ok(_) => (),
        Err(_) => {
            set_text("Failed to launch!");
            return false;
        }
    }

    set_text("Done!");
    tokio::time::sleep(Duration::from_millis(1000)).await;

    info!("Install Successful!");

    return true;
}
