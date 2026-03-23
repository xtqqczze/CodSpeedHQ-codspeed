use axoupdater::AxoUpdater;

use crate::prelude::*;

pub async fn run() -> Result<()> {
    let mut updater = AxoUpdater::new_for("codspeed-runner");
    if let Err(e) = updater.load_receipt() {
        debug!("Failed to load update receipt: {e}");
        bail!(
            "Please re-install codspeed by following https://codspeed.io/docs/cli#installation to enable self updates"
        );
    };

    let result = updater.run().await?;
    match result {
        Some(outcome) => {
            info!("Updated codspeed to version {}", outcome.new_version_tag);
        }
        None => {
            info!("codspeed is already up to date");
        }
    }
    Ok(())
}
