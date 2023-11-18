use std::process::{exit, Stdio};

#[cfg(target_os = "windows")]
const ISOBIN_BIN: &str = "isobin.exe";

#[cfg(not(target_os = "windows"))]
const ISOBIN_BIN: &str = "isobin";

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let current_exe = std::env::current_exe().unwrap();
    let current_dir = current_exe.parent().unwrap();
    let isobin_path = current_dir.join(ISOBIN_BIN);
    let mut command_args = vec!["run".into()];
    command_args.extend_from_slice(&args[1..]);

    if isobin_path.exists() {
        let status = std::process::Command::new(isobin_path)
            .args(command_args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .unwrap();
        if !status.success() {
            if let Some(code) = status.code() {
                exit(code);
            } else {
                exit(1);
            }
        }
    } else {
        eprintln!("isobin \"{}\" is not found", isobin_path.display());
    }
}
