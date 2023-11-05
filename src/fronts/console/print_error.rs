use colored::Colorize;

use crate::{Error, InstallServiceError, IsobinConfigError};
pub fn print_error(err: &Error) {
    eprintln!();
    eprintln!();
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
            eprintln!("An error occurred in {provider}/{name}.");
            eprintln!("{}", error_message.red());
        }
        _ => match err.downcast_ref::<IsobinConfigError>() {
            Some(IsobinConfigError::MultiValidate(errs)) => {
                for err in errs.iter() {
                    print_error(err);
                }
            }
            Some(IsobinConfigError::Validate {
                provider,
                name,
                error,
            }) => {
                eprintln!("Invalid config value in {provider}/{name}.");
                eprintln!("{}", error.to_string().red());
            }
            _ => {
                eprintln!("{}", err.to_string().red());
            }
        },
    }
}
