use std::{borrow::Cow, time::Duration};

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    providers::{ProviderKind, TargetDependency},
    Result,
};

#[derive(Clone)]
pub struct Progress {
    progress_bar: ProgressBar,
    provider_kind: ProviderKind,
    name: String,
    summary: String,
}

impl Progress {
    pub fn new(progress_bar: ProgressBar, install_target: &impl TargetDependency) -> Self {
        Self {
            progress_bar,
            provider_kind: install_target.provider_kind(),
            name: install_target.name().into(),
            summary: install_target.summary(),
        }
    }

    fn start(&self, prefix: impl Into<Cow<'static, str>>) -> Result<()> {
        self.progress_bar
            .enable_steady_tick(Duration::from_millis(100));
        self.progress_bar.set_prefix(prefix);
        Ok(())
    }

    fn done(&self, prefix: impl Into<Cow<'static, str>>) -> Result<()> {
        self.progress_bar.disable_steady_tick();
        self.progress_bar
            .set_style(ProgressStyle::with_template("  {prefix} {msg}")?);
        self.progress_bar.set_prefix(prefix.into());
        self.progress_bar.finish();
        Ok(())
    }

    pub fn failed(&self, prefix: impl AsRef<str>) -> Result<()> {
        self.progress_bar.disable_steady_tick();
        self.progress_bar
            .set_style(ProgressStyle::with_template("  {prefix} {msg}")?);
        self.progress_bar
            .set_prefix(prefix.as_ref().red().to_string());
        self.progress_bar.finish();
        Ok(())
    }
}

impl crate::fronts::Progress for Progress {
    fn failed_install(&self) -> Result<()> {
        self.failed("failed install")
    }

    fn failed_uninstall(&self) -> Result<()> {
        self.failed("failed uninstall")
    }

    fn prepare_install(&self) -> Result<()> {
        self.progress_bar
            .set_style(ProgressStyle::with_template("{spinner} {prefix} {msg}")?);
        self.progress_bar.set_prefix("waiting");
        self.progress_bar.set_message(format!(
            "{}/{} {}",
            self.provider_kind, self.name, self.summary
        ));
        Ok(())
    }
    fn ready_install(&self) -> Result<()> {
        self.done("ready install".green().to_string())
    }
    fn ready_uninstall(&self) -> Result<()> {
        self.done("ready uninstall".yellow().to_string())
    }
    fn done_install(&self) -> Result<()> {
        self.done("done install".green().to_string())
    }

    fn done_uninstall(&self) -> Result<()> {
        self.done("done uninstall".yellow().to_string())
    }

    fn start_uninstall(&self) -> Result<()> {
        self.start("uninstalling".yellow().to_string())
    }

    fn start_install(&self) -> Result<()> {
        self.start("installing".magenta().to_string())
    }

    fn already_installed(&self) -> Result<()> {
        self.progress_bar
            .set_prefix("already installed".cyan().to_string());
        self.progress_bar.finish();
        Ok(())
    }
}
