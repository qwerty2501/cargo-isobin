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

    pub fn prepare(&self) -> Result<()> {
        self.progress_bar
            .set_style(ProgressStyle::with_template("{spinner} {prefix} {msg}")?);
        self.progress_bar
            .set_prefix(format!("{}/{}", self.provider_kind, self.name));
        self.progress_bar.set_message("waiting");
        Ok(())
    }

    pub fn start(&self) -> Result<()> {
        self.progress_bar
            .enable_steady_tick(Duration::from_millis(100));
        self.progress_bar.set_message("installing");
        Ok(())
    }

    pub fn done(&self) -> Result<()> {
        self.progress_bar.disable_steady_tick();
        self.progress_bar
            .set_style(ProgressStyle::with_template("  {prefix} {msg}")?);
        self.progress_bar.set_message("done");
        self.progress_bar.finish();
        Ok(())
    }

    pub fn failed(&self) -> Result<()> {
        self.progress_bar.disable_steady_tick();
        self.progress_bar
            .set_style(ProgressStyle::with_template("  {prefix} {msg}")?);
        self.progress_bar
            .set_message(format!("{}/{} failed", self.provider_kind, self.name));
        self.progress_bar.finish();
        Ok(())
    }
}
