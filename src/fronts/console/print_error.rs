use colored::Colorize;

use crate::{install::InstallServiceError, Error, IsobinManifestError, ProviderKind};
pub fn print_error(err: &Error) {
    match err.downcast_ref::<InstallServiceError>() {
        Some(InstallServiceError::MultiInstall(errs)) => {
            for err in errs.iter() {
                print_error(err);
            }
        }
        Some(InstallServiceError::Install {
            provider,
            name,
            error_message,
            error: _,
        }) => {
            eprintln!(
                "An error occurred in {}/{}.",
                provider.to_string().red(),
                name.red()
            );
            print_provider_error(provider, error_message);
        }
        _ => match err.downcast_ref::<IsobinManifestError>() {
            Some(IsobinManifestError::MultiValidate(errs)) => {
                for err in errs.iter() {
                    print_error(err);
                }
            }
            Some(IsobinManifestError::Validate {
                provider,
                name,
                error,
            }) => {
                eprintln!(
                    "Invalid config value in {}/{}.",
                    provider.to_string().red(),
                    name.red()
                );
                eprintln!("{}", error.to_string().red());
            }
            _ => {
                eprintln!("{}", err.to_string().red());
            }
        },
    }
}

fn print_provider_error(provider: &ProviderKind, message: &str) {
    match provider {
        ProviderKind::Cargo => eprintln!("{message}"),
    }
}
