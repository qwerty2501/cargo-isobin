use indicatif::{ProgressBar, ProgressStyle};

use crate::{providers::InstallTarget, Result};

#[derive(new, Getters, Clone)]
pub struct Progress {
    progress_bar: ProgressBar,
}

impl Progress {
    pub fn start(&self, install_target: &impl InstallTarget) -> Result<()> {
        self.progress_bar
            .set_style(ProgressStyle::with_template("{spinner} {msg}")?);
        self.progress_bar.set_message(format!(
            "{}/{}",
            install_target.provider_kind(),
            install_target.name()
        ));
        Ok(())
    }

    pub fn done(&self, install_target: &impl InstallTarget) -> Result<()> {
        self.progress_bar
            .set_style(ProgressStyle::with_template("{msg.green}")?);
        self.progress_bar.set_message(format!(
            "{}/{} done",
            install_target.provider_kind(),
            install_target.name()
        ));
        Ok(())
    }
    pub fn failed(&self, install_target: &impl InstallTarget) -> Result<()> {
        self.progress_bar
            .set_style(ProgressStyle::with_template("{msg.red}")?);
        self.progress_bar.set_message(format!(
            "{}/{} failed",
            install_target.provider_kind(),
            install_target.name()
        ));
        Ok(())
    }
}
