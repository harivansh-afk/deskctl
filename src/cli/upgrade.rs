use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use serde_json::json;

use crate::cli::GlobalOpts;
use crate::core::protocol::Response;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InstallMethod {
    Npm,
    Cargo,
    Nix,
    Source,
    Unknown,
}

impl InstallMethod {
    fn as_str(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Cargo => "cargo",
            Self::Nix => "nix",
            Self::Source => "source",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug)]
struct UpgradePlan {
    install_method: InstallMethod,
    program: &'static str,
    args: Vec<&'static str>,
}

impl UpgradePlan {
    fn command_line(&self) -> String {
        std::iter::once(self.program)
            .chain(self.args.iter().copied())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub fn run_upgrade(opts: &GlobalOpts) -> Result<Response> {
    let current_exe = std::env::current_exe().context("Failed to determine executable path")?;
    let install_method = detect_install_method(&current_exe);

    let Some(plan) = upgrade_plan(install_method) else {
        return Ok(Response::err_with_data(
            format!(
                "deskctl upgrade is not supported for {} installs.",
                install_method.as_str()
            ),
            json!({
                "kind": "upgrade_unsupported",
                "install_method": install_method.as_str(),
                "current_exe": current_exe.display().to_string(),
                "hint": upgrade_hint(install_method),
            }),
        ));
    };

    if opts.json {
        let output = match Command::new(plan.program).args(&plan.args).output() {
            Ok(output) => output,
            Err(error) => return Ok(upgrade_spawn_error_response(&plan, &error)),
        };

        return Ok(upgrade_response(
            &plan,
            output.status.code(),
            output.status.success(),
        ));
    }

    let status = match Command::new(plan.program)
        .args(&plan.args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
    {
        Ok(status) => status,
        Err(error) => return Ok(upgrade_spawn_error_response(&plan, &error)),
    };

    Ok(upgrade_response(&plan, status.code(), status.success()))
}

fn upgrade_response(plan: &UpgradePlan, exit_code: Option<i32>, success: bool) -> Response {
    let data = json!({
        "action": "upgrade",
        "install_method": plan.install_method.as_str(),
        "command": plan.command_line(),
        "exit_code": exit_code,
    });

    if success {
        Response::ok(data)
    } else {
        Response::err_with_data(
            format!("Upgrade command failed: {}", plan.command_line()),
            json!({
                "kind": "upgrade_failed",
                "install_method": plan.install_method.as_str(),
                "command": plan.command_line(),
                "exit_code": exit_code,
                "hint": upgrade_hint(plan.install_method),
            }),
        )
    }
}

fn upgrade_spawn_error_response(plan: &UpgradePlan, error: &std::io::Error) -> Response {
    Response::err_with_data(
        format!("Failed to run {}", plan.command_line()),
        json!({
            "kind": "upgrade_failed",
            "install_method": plan.install_method.as_str(),
            "command": plan.command_line(),
            "io_error": error.to_string(),
            "hint": upgrade_hint(plan.install_method),
        }),
    )
}

fn upgrade_plan(install_method: InstallMethod) -> Option<UpgradePlan> {
    match install_method {
        InstallMethod::Npm => Some(UpgradePlan {
            install_method,
            program: "npm",
            args: vec!["install", "-g", "deskctl@latest"],
        }),
        InstallMethod::Cargo => Some(UpgradePlan {
            install_method,
            program: "cargo",
            args: vec!["install", "deskctl", "--locked"],
        }),
        InstallMethod::Nix | InstallMethod::Source | InstallMethod::Unknown => None,
    }
}

fn upgrade_hint(install_method: InstallMethod) -> &'static str {
    match install_method {
        InstallMethod::Nix => {
            "Use nix profile upgrade or update the flake reference you installed from."
        }
        InstallMethod::Source => {
            "Rebuild from source or reinstall deskctl through npm, cargo, or nix."
        }
        InstallMethod::Unknown => {
            "Reinstall deskctl through a supported channel such as npm, cargo, or nix."
        }
        InstallMethod::Npm => "Retry the npm install command directly if the upgrade failed.",
        InstallMethod::Cargo => {
            "Retry cargo install directly or confirm the crate is published for this version."
        }
    }
}

fn detect_install_method(current_exe: &Path) -> InstallMethod {
    if looks_like_npm_install(current_exe) {
        return InstallMethod::Npm;
    }
    if looks_like_nix_install(current_exe) {
        return InstallMethod::Nix;
    }
    if looks_like_cargo_install(current_exe) {
        return InstallMethod::Cargo;
    }
    if looks_like_source_tree(current_exe) {
        return InstallMethod::Source;
    }
    InstallMethod::Unknown
}

fn looks_like_npm_install(path: &Path) -> bool {
    let value = normalize(path);
    value.contains("/node_modules/deskctl/") && value.contains("/vendor/")
}

fn looks_like_nix_install(path: &Path) -> bool {
    normalize(path).starts_with("/nix/store/")
}

fn looks_like_cargo_install(path: &Path) -> bool {
    let Some(home) = std::env::var_os("HOME") else {
        return false;
    };

    let cargo_home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(home).join(".cargo"));
    path == cargo_home.join("bin").join("deskctl")
}

fn looks_like_source_tree(path: &Path) -> bool {
    let value = normalize(path);
    value.contains("/target/debug/deskctl") || value.contains("/target/release/deskctl")
}

fn normalize(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{detect_install_method, upgrade_plan, InstallMethod};

    #[test]
    fn detects_npm_install_path() {
        let method = detect_install_method(Path::new(
            "/usr/local/lib/node_modules/deskctl/vendor/deskctl-linux-x86_64",
        ));
        assert_eq!(method, InstallMethod::Npm);
    }

    #[test]
    fn detects_nix_install_path() {
        let method = detect_install_method(Path::new("/nix/store/abc123-deskctl/bin/deskctl"));
        assert_eq!(method, InstallMethod::Nix);
    }

    #[test]
    fn detects_source_tree_path() {
        let method =
            detect_install_method(Path::new("/Users/example/src/deskctl/target/debug/deskctl"));
        assert_eq!(method, InstallMethod::Source);
    }

    #[test]
    fn npm_upgrade_plan_uses_global_install() {
        let plan = upgrade_plan(InstallMethod::Npm).expect("npm installs should support upgrade");
        assert_eq!(plan.command_line(), "npm install -g deskctl@latest");
    }

    #[test]
    fn nix_install_has_no_upgrade_plan() {
        assert!(upgrade_plan(InstallMethod::Nix).is_none());
    }
}
