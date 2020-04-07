use std::env;
use std::io;
#[cfg(target_family = "unix")]
use std::fs;
#[cfg(target_family = "unix")]
use std::fs::Permissions;
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

pub fn execute(target: &Path) -> io::Result<i32> {
    trace!("target={:?}", target);

    let args: Vec<String> = env::args().skip(1).collect();
    trace!("args={:?}", args);

    do_execute(target, &args)
}

#[cfg(target_family = "unix")]
fn ensure_executable(target: &Path) -> io::Result<()> {
    add_exec_permission(target, true, true, false)?;
    Ok(())
}

#[cfg(target_family = "unix")]
fn add_exec_permission(file: &Path, user: bool, group: bool, other: bool) -> io::Result<()> {
    let permissions = file.metadata()?.permissions();
    let mode = permissions.mode();

    let mut new_mode = mode;
    if user {
        new_mode |= 1 << 6;
    }
    if group {
        new_mode |= 1 << 3;
    }
    if other {
        new_mode |= 1
    }

    fs::set_permissions(file, Permissions::from_mode(new_mode))?;

    Ok(())
}

#[cfg(target_family = "unix")]
fn do_execute(target: &Path, args: &[String]) -> io::Result<i32> {
    ensure_executable(target)?;

    Ok(Command::new(target)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("WARP_EXEC_PATH", env::current_exe()?)
        .spawn()?
        .wait()?
        .code().unwrap_or(1))
}

#[cfg(target_family = "windows")]
fn is_script(target: &Path) -> bool {
    const SCRIPT_EXTENSIONS: &[&str] = &["bat", "cmd"];
    SCRIPT_EXTENSIONS.contains(
        &target.extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase().as_str())
}

#[cfg(target_family = "windows")]
fn is_vbs(target: &Path) -> bool {
    const VBS_EXTENSIONS: &[&str] = &["vbs"];
    VBS_EXTENSIONS.contains(
        &target.extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase().as_str())
}

#[cfg(target_family = "windows")]
fn do_execute(target: &Path, args: &[String]) -> io::Result<i32> {
    let target_str = target.as_os_str().to_str().unwrap();

    if is_script(target) {
        let mut cmd_args = Vec::with_capacity(args.len() + 2);
        cmd_args.push("/c".to_string());
        cmd_args.push(target_str.to_string());
        cmd_args.extend_from_slice(&args);

        Ok(Command::new("cmd")
            .args(cmd_args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env("WARP_EXEC_PATH", env::current_exe()?)
            .spawn()?
            .wait()?
            .code().unwrap_or(1))
    } else if is_vbs(target) {
        let mut cmd_args = Vec::with_capacity(args.len() + 2);
        cmd_args.push("/nologo".to_string());
        cmd_args.push(target_str.to_string());
        cmd_args.extend_from_slice(&args);

        Ok(Command::new("wscript")
            .args(cmd_args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env("WARP_EXEC_PATH", env::current_exe()?)
            .spawn()?
            .wait()?
            .code().unwrap_or(1))
    } else {
        Ok(Command::new(target)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env("WARP_EXEC_PATH", env::current_exe()?)
            .spawn()?
            .wait()?
            .code().unwrap_or(1))
    }
}