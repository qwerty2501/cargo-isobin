use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    providers::{InstallTarget, ProviderKind},
    Result,
};

#[derive(Clone)]
pub struct Progress {
    progress_bar: ProgressBar,
    provider_kind: ProviderKind,
    name: String,
}

impl Progress {
    pub fn new(progress_bar: ProgressBar, install_target: &impl InstallTarget) -> Self {
        Self {
            progress_bar,
            provider_kind: install_target.provider_kind(),
            name: install_target.name().into(),
        }
    }
    pub fn start(&self) -> Result<()> {
        self.progress_bar
            .set_style(ProgressStyle::with_template("{spinner} {msg}")?);
        self.progress_bar
            .enable_steady_tick(Duration::from_millis(100));
        self.progress_bar
            .set_message(format!("{}/{} installing", self.provider_kind, self.name));
        Ok(())
    }

    pub fn done(&self) -> Result<()> {
        self.progress_bar
            .set_style(ProgressStyle::with_template("{msg.green}")?);
        self.progress_bar
            .set_message(format!("{}/{} done", self.provider_kind, self.name));
        Ok(())
    }
    pub fn failed(&self) -> Result<()> {
        self.progress_bar
            .set_style(ProgressStyle::with_template("{msg.red}")?);
        self.progress_bar
            .set_message(format!("{}/{} failed", self.provider_kind, self.name));
        Ok(())
    }
}
