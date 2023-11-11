#[derive(Clone)]
pub struct Progress;

impl crate::fronts::Progress for Progress {
    fn ready_uninstall(&self) -> crate::Result<()> {
        Ok(())
    }
    fn ready_install(&self) -> crate::Result<()> {
        Ok(())
    }
    fn done_install(&self) -> crate::Result<()> {
        Ok(())
    }
    fn start_install(&self) -> crate::Result<()> {
        Ok(())
    }
    fn done_uninstall(&self) -> crate::Result<()> {
        Ok(())
    }
    fn failed_install(&self) -> crate::Result<()> {
        Ok(())
    }
    fn prepare_install(&self) -> crate::Result<()> {
        Ok(())
    }
    fn start_uninstall(&self) -> crate::Result<()> {
        Ok(())
    }
    fn failed_uninstall(&self) -> crate::Result<()> {
        Ok(())
    }
    fn already_installed(&self) -> crate::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct MultiProgress;

impl crate::fronts::MultiProgress for MultiProgress {
    type Progress = Progress;
    fn make_progress(&self, _: &impl crate::providers::InstallTarget) -> Self::Progress {
        Self::Progress {}
    }
}
