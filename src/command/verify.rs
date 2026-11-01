use log::info;

use crate::{
    cli::Args,
    command::install_utils::{verify_release, ReleaseDownload},
};

pub fn verify<F>(args: Args, set_text: F) -> bool
where
    F: Fn(&str) -> (),
{
    info!("Starting installation");

    set_text("Verifying release");

    if let Some(file) = args.build {
        let bytes = std::fs::read(&file).unwrap();

        if let Some(sig) = args.signature {
            let sig = std::fs::read(&sig).unwrap();

            let file = ReleaseDownload {
                signature: sig,
                binary: bytes,
            };

            set_text("Read files");

            if verify_release(&file) == false {
                set_text("Failed to verify release...");
                return false;
            } else {
                set_text("Verified release!")
            }
        } else {
            set_text("No signature")
        }
    } else {
        set_text("No file to verify")
    }

    return false;
}
