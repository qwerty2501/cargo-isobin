pub mod console;
pub mod quiet;

use crate::providers::TargetDependency;
use crate::Result;
pub use console::print_error;

pub trait MultiProgress: Clone + 'static + Send + Sync + Default {
    type Progress: Progress;
    fn make_progress(&self, install_target: &impl TargetDependency) -> Self::Progress;
}
pub trait Progress: Clone + 'static + Send + Sync {
    fn prepare_install(&self) -> Result<()>;
    fn already_installed(&self) -> Result<()>;
    fn start_uninstall(&self) -> Result<()>;
    fn start_install(&self) -> Result<()>;
    fn ready_install(&self) -> Result<()>;
    fn ready_uninstall(&self) -> Result<()>;
    fn done_install(&self) -> Result<()>;
    fn done_uninstall(&self) -> Result<()>;
    fn failed_install(&self) -> Result<()>;
    fn failed_uninstall(&self) -> Result<()>;
}
