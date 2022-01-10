use std::marker::PhantomData;

use super::*;
type Result<T> = std::result::Result<T, InstallError>;
#[async_trait]
pub trait Installer: 'static + Send + Sync {
    type InstallTarget: InstallTarget;
    type MultiInstaller: MultiInstaller<Installer = Self>;

    async fn installs(&self, targets: &[Self::InstallTarget]) -> Result<()> {
        Self::MultiInstaller::installs(self, targets).await
    }
    fn provider_type(&self) -> providers::ProviderKind;
    async fn install(&self, target: &Self::InstallTarget) -> Result<()> {
        target.install().await
    }
}

#[async_trait]
pub trait MultiInstaller {
    type Installer: Installer;
    async fn installs(
        installer: &Self::Installer,
        targets: &[<Self::Installer as Installer>::InstallTarget],
    ) -> Result<()>;
}

pub struct SequenceMultiInstaller<I>(PhantomData<I>);

#[async_trait]
impl<I: Installer> MultiInstaller for SequenceMultiInstaller<I> {
    type Installer = I;
    async fn installs(installer: &I, targets: &[I::InstallTarget]) -> Result<()>
    where
        I: Installer,
    {
        for target in targets.iter() {
            installer.install(target).await?;
        }
        Ok(())
    }
}

#[async_trait]
pub trait InstallTarget: 'static + Send + Sync {
    fn provider_type(&self) -> providers::ProviderKind;

    async fn install(&self) -> Result<()>;
}

#[derive(thiserror::Error, Debug, new)]
pub enum InstallError {
    #[error("todo")]
    Todo(),
}
