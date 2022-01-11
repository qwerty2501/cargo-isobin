use super::*;
pub struct CargoInstaller {}
pub struct CargoInstallTarget {}

type Result<T> = std::result::Result<T, InstallError>;
#[async_trait]
impl providers::Installer for CargoInstaller {
    type InstallTarget = CargoInstallTarget;
    type MultiInstaller = ParallelMultiInstaller<Self>;

    fn provider_type(&self) -> providers::ProviderKind {
        providers::ProviderKind::Cargo
    }
}

#[async_trait]
impl providers::InstallTarget for CargoInstallTarget {
    fn provider_type(&self) -> providers::ProviderKind {
        providers::ProviderKind::Cargo
    }
    async fn install(&self) -> Result<()> {
        todo!()
    }
}
