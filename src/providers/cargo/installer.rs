use super::*;
pub struct CargoInstaller {}
pub struct CargoInstallTarget {}

type Result<T> = std::result::Result<T, InstallError>;
#[async_trait]
impl providers::Installer for CargoInstaller {
    type InstallTarget = CargoInstallTarget;

    fn provider_kind(&self) -> providers::ProviderKind {
        providers::ProviderKind::Cargo
    }
    fn multi_install_mode(&self) -> providers::MultiInstallMode {
        providers::MultiInstallMode::Parallel
    }

    #[allow(unused_variables)]
    async fn install(&self, target: &Self::InstallTarget) -> Result<()> {
        todo!()
    }
}

#[async_trait]
impl providers::InstallTarget for CargoInstallTarget {}
