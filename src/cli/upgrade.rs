use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use serde_json::json;

use crate::cli::{GlobalOpts, UpgradeOpts};
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

#[derive(Debug)]
struct VersionInfo {
    current: String,
    latest: String,
}

pub fn run_upgrade(opts: &GlobalOpts, upgrade_opts: &UpgradeOpts) -> Result<Response> {
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

    if !opts.json {
        println!("- Checking for updates...");
    }

    let versions = match resolve_versions(&plan) {
        Ok(versions) => versions,
        Err(response) => return Ok(response),
    };

    if versions.current == versions.latest {
        return Ok(Response::ok(json!({
            "action": "upgrade",
            "status": "up_to_date",
            "install_method": plan.install_method.as_str(),
            "current_version": versions.current,
            "latest_version": versions.latest,
        })));
    }

    if !upgrade_opts.yes {
        if opts.json || !io::stdin().is_terminal() {
            return Ok(Response::err_with_data(
                format!(
                    "Upgrade confirmation required for {} -> {}.",
                    versions.current, versions.latest
                ),
                json!({
                    "kind": "upgrade_confirmation_required",
                    "install_method": plan.install_method.as_str(),
                    "current_version": versions.current,
                    "latest_version": versions.latest,
                    "command": plan.command_line(),
                    "hint": "Re-run with --yes to upgrade non-interactively.",
                }),
            ));
        }

        if !confirm_upgrade(&versions)? {
            return Ok(Response::ok(json!({
                "action": "upgrade",
                "status": "cancelled",
                "install_method": plan.install_method.as_str(),
                "current_version": versions.current,
                "latest_version": versions.latest,
            })));
        }
    }

    if !opts.json {
        println!(
            "- Upgrading deskctl from {} -> {}...",
            versions.current, versions.latest
        );
    }

    let output = match Command::new(plan.program).args(&plan.args).output() {
        Ok(output) => output,
        Err(error) => return Ok(upgrade_spawn_error_response(&plan, &versions, &error)),
    };

    if output.status.success() {
        return Ok(Response::ok(json!({
            "action": "upgrade",
            "status": "upgraded",
            "install_method": plan.install_method.as_str(),
            "current_version": versions.current,
            "latest_version": versions.latest,
            "command": plan.command_line(),
            "exit_code": output.status.code(),
        })));
    }

    Ok(upgrade_command_failed_response(&plan, &versions, &output))
}

fn resolve_versions(plan: &UpgradePlan) -> std::result::Result<VersionInfo, Response> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let latest = match plan.install_method {
        InstallMethod::Npm => query_npm_latest_version()?,
        InstallMethod::Cargo => query_cargo_latest_version()?,
        InstallMethod::Nix | InstallMethod::Source | InstallMethod::Unknown => {
            return Err(Response::err_with_data(
                "Could not determine the latest published version.".to_string(),
                json!({
                    "kind": "upgrade_failed",
                    "install_method": plan.install_method.as_str(),
                    "reason": "Could not determine the latest published version for this install method.",
                    "command": plan.command_line(),
                    "hint": upgrade_hint(plan.install_method),
                }),
            ));
        }
    };

    Ok(VersionInfo { current, latest })
}

fn query_npm_latest_version() -> std::result::Result<String, Response> {
    let output = Command::new("npm")
        .args(["view", "deskctl", "version", "--json"])
        .output()
        .map_err(|error| {
            Response::err_with_data(
                "Failed to check the latest npm version.".to_string(),
                json!({
                    "kind": "upgrade_failed",
                    "install_method": InstallMethod::Npm.as_str(),
                    "reason": "Failed to run npm view deskctl version --json.",
                    "io_error": error.to_string(),
                    "command": "npm view deskctl version --json",
                    "hint": upgrade_hint(InstallMethod::Npm),
                }),
            )
        })?;

    if !output.status.success() {
        return Err(Response::err_with_data(
            "Failed to check the latest npm version.".to_string(),
            json!({
                "kind": "upgrade_failed",
                "install_method": InstallMethod::Npm.as_str(),
                "reason": command_failure_reason(&output),
                "command": "npm view deskctl version --json",
                "hint": upgrade_hint(InstallMethod::Npm),
            }),
        ));
    }

    serde_json::from_slice::<String>(&output.stdout).map_err(|_| {
        Response::err_with_data(
            "Failed to parse the latest npm version.".to_string(),
            json!({
                "kind": "upgrade_failed",
                "install_method": InstallMethod::Npm.as_str(),
                "reason": "npm view returned an unexpected version payload.",
                "command": "npm view deskctl version --json",
                "hint": upgrade_hint(InstallMethod::Npm),
            }),
        )
    })
}

fn query_cargo_latest_version() -> std::result::Result<String, Response> {
    let output = Command::new("cargo")
        .args(["search", "deskctl", "--limit", "1"])
        .output()
        .map_err(|error| {
            Response::err_with_data(
                "Failed to check the latest crates.io version.".to_string(),
                json!({
                    "kind": "upgrade_failed",
                    "install_method": InstallMethod::Cargo.as_str(),
                    "reason": "Failed to run cargo search deskctl --limit 1.",
                    "io_error": error.to_string(),
                    "command": "cargo search deskctl --limit 1",
                    "hint": upgrade_hint(InstallMethod::Cargo),
                }),
            )
        })?;

    if !output.status.success() {
        return Err(Response::err_with_data(
            "Failed to check the latest crates.io version.".to_string(),
            json!({
                "kind": "upgrade_failed",
                "install_method": InstallMethod::Cargo.as_str(),
                "reason": command_failure_reason(&output),
                "command": "cargo search deskctl --limit 1",
                "hint": upgrade_hint(InstallMethod::Cargo),
            }),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let latest = stdout
        .split('"')
        .nth(1)
        .map(str::to_string)
        .filter(|value| !value.is_empty());

    latest.ok_or_else(|| {
        Response::err_with_data(
            "Failed to determine the latest crates.io version.".to_string(),
            json!({
                "kind": "upgrade_failed",
                "install_method": InstallMethod::Cargo.as_str(),
                "reason": "cargo search did not return a published deskctl crate version.",
                "command": "cargo search deskctl --limit 1",
                "hint": upgrade_hint(InstallMethod::Cargo),
            }),
        )
    })
}

fn confirm_upgrade(versions: &VersionInfo) -> Result<bool> {
    print!(
        "Upgrade deskctl from {} -> {}? [y/N] ",
        versions.current, versions.latest
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    Ok(matches!(trimmed, "y" | "Y" | "yes" | "YES" | "Yes"))
}

fn upgrade_command_failed_response(
    plan: &UpgradePlan,
    versions: &VersionInfo,
    output: &std::process::Output,
) -> Response {
    Response::err_with_data(
        format!("Upgrade command failed: {}", plan.command_line()),
        json!({
            "kind": "upgrade_failed",
            "install_method": plan.install_method.as_str(),
            "current_version": versions.current,
            "latest_version": versions.latest,
            "command": plan.command_line(),
            "exit_code": output.status.code(),
            "reason": command_failure_reason(output),
            "hint": upgrade_hint(plan.install_method),
        }),
    )
}

fn upgrade_spawn_error_response(
    plan: &UpgradePlan,
    versions: &VersionInfo,
    error: &std::io::Error,
) -> Response {
    Response::err_with_data(
        format!("Failed to run {}", plan.command_line()),
        json!({
            "kind": "upgrade_failed",
            "install_method": plan.install_method.as_str(),
            "current_version": versions.current,
            "latest_version": versions.latest,
            "command": plan.command_line(),
            "io_error": error.to_string(),
            "hint": upgrade_hint(plan.install_method),
        }),
    )
}

fn command_failure_reason(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    stderr
        .lines()
        .chain(stdout.lines())
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            output
                .status
                .code()
                .map(|code| format!("Command exited with status {code}."))
                .unwrap_or_else(|| "Command exited unsuccessfully.".to_string())
        })
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
        InstallMethod::Npm => "Retry with --yes or run npm install -g deskctl@latest directly.",
        InstallMethod::Cargo => "Retry with --yes or run cargo install deskctl --locked directly.",
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
    use std::os::unix::process::ExitStatusExt;
    use std::path::Path;

    use super::{command_failure_reason, detect_install_method, upgrade_plan, InstallMethod};

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

    #[test]
    fn failure_reason_prefers_stderr() {
        let output = std::process::Output {
            status: std::process::ExitStatus::from_raw(1 << 8),
            stdout: b"".to_vec(),
            stderr: b"boom\n".to_vec(),
        };

        assert_eq!(command_failure_reason(&output), "boom");
    }
}
