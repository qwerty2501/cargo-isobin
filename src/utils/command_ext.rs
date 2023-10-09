use std::process::ExitStatus;

use tokio::process::Command;

use super::*;

#[derive(thiserror::Error, Debug, new)]
#[error("{stderr}")]
pub struct RunCommandError {
    exit_status: ExitStatus,
    stdout: String,
    stderr: String,
}

pub async fn run_install_command(
    installer_name: impl Into<String>,
    target_name: impl Into<String>,
    command: Command,
) -> Result<()> {
    run_commnad(command).await.map_err(|err| {
        InstallServiceError::new_install(installer_name.into(), target_name.into(), Box::new(err))
            .into()
    })
}

pub async fn run_commnad(mut command: Command) -> Result<()> {
    match command.output().await {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                Err(RunCommandError::new(output.status, stdout.into(), stderr.into()).into())
            }
        }
        Err(err) => Err(crate::errors::Error::new_fatal(err.into())),
    }
}
