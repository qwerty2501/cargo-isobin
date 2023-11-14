use indicatif::{MultiProgress as IndicatifMultiProgress, ProgressBar as IndicatifProgressBar};

use crate::providers::TargetDependency;

use super::Progress;

#[derive(Default, Clone)]
pub struct MultiProgress {
    multi_progress: IndicatifMultiProgress,
}

impl crate::fronts::MultiProgress for MultiProgress {
    type Progress = Progress;
    fn make_progress(&self, install_target: &impl TargetDependency) -> Self::Progress {
        Progress::new(
            self.multi_progress.add(IndicatifProgressBar::hidden()),
            install_target,
        )
    }
}
