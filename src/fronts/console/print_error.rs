use colored::Colorize;

use crate::{Error, InstallServiceError};
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
        Some(_) | None => {
            eprintln!("{}", err.to_string().red());
        }
    }
}
