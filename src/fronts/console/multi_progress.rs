use indicatif::MultiProgress as IndicatifMultiProgress;

use super::Progress;

#[derive(new, Clone)]
pub struct MultiProgress {
    multi_progress: IndicatifMultiProgress,
    progresses: Vec<Progress>,
}
