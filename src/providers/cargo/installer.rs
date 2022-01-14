use super::*;

#[derive(Default)]
pub struct CargoCoreInstaller {}

#[async_trait]
impl providers::CoreInstaller for CargoCoreInstaller {
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

#[derive(new, Getters)]
pub struct CargoInstallTarget {
    name: String,
    install_dependency: CargoInstallDependency,
}

#[async_trait]
impl providers::InstallTarget for CargoInstallTarget {}
