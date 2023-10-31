use indicatif::{MultiProgress as IndicatifMultiProgress, ProgressBar as IndicatifProgressBar};

use crate::providers::InstallTarget;

use super::Progress;

#[derive(Default, Clone)]
pub struct MultiProgress {
    multi_progress: IndicatifMultiProgress,
}

impl MultiProgress {
    pub fn make_progress(&self, install_target: &impl InstallTarget) -> Progress {
        Progress::new(
            self.multi_progress.add(IndicatifProgressBar::hidden()),
            install_target,
        )
    }
}
