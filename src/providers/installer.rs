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

pub struct ParallelMultiInstaller<I>(PhantomData<I>);

#[async_trait]
impl<I: Installer> MultiInstaller for ParallelMultiInstaller<I> {
    type Installer = I;
    async fn installs(installer: &I, targets: &[I::InstallTarget]) -> Result<()>
    where
        I: Installer,
    {
        let mut target_futures = Vec::with_capacity(targets.len());
        for target in targets.iter() {
            target_futures.push(installer.install(target));
        }
        let mut target_errors = vec![];
        for target_future in target_futures.into_iter() {
            let result = target_future.await;
            if let Some(err) = result.err() {
                target_errors.push(err);
            }
        }
        if !target_errors.is_empty() {
            Err(InstallError::Multi(target_errors))
        } else {
            Ok(())
        }
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
    #[error("multi error")]
    Multi(Vec<InstallError>),
}
