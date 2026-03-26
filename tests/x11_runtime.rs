#![cfg(target_os = "linux")]

mod support;

use anyhow::Result;
use deskctl::cli::connection::send_command;
use deskctl::core::doctor;
use deskctl::core::protocol::Request;

use self::support::{
    deskctl_tmp_screenshot_count, env_lock, json_response, successful_json_response, FixtureWindow,
    SessionEnvGuard, TestSession,
};

#[test]
fn doctor_reports_healthy_x11_environment() -> Result<()> {
    let _guard = env_lock().lock().unwrap();
    let Some(_env) = SessionEnvGuard::prepare() else {
        eprintln!("Skipping X11 integration test because DISPLAY is not set");
        return Ok(());
    };

    let _window = FixtureWindow::create("deskctl doctor test", "DeskctlDoctor")?;
    let session = TestSession::new("doctor")?;
    let report = doctor::run(session.socket_path());

    assert!(report
        .checks
        .iter()
        .any(|check| check.name == "display" && check.ok));
    assert!(report
        .checks
        .iter()
        .any(|check| check.name == "backend" && check.ok));
    assert!(report
        .checks
        .iter()
        .any(|check| check.name == "window-enumeration" && check.ok));
    assert!(report
        .checks
        .iter()
        .any(|check| check.name == "screenshot" && check.ok));

    Ok(())
}

#[test]
fn list_windows_is_side_effect_free() -> Result<()> {
    let _guard = env_lock().lock().unwrap();
    let Some(_env) = SessionEnvGuard::prepare() else {
        eprintln!("Skipping X11 integration test because DISPLAY is not set");
        return Ok(());
    };

    let _window = FixtureWindow::create("deskctl list-windows test", "DeskctlList")?;
    let session = TestSession::new("list-windows")?;
    session.start_daemon_cli()?;

    let before = deskctl_tmp_screenshot_count();
    let response = send_command(&session.opts, &Request::new("list-windows"))?;
    assert!(response.success);

    let windows = response
        .data
        .and_then(|data| data.get("windows").cloned())
        .and_then(|windows| windows.as_array().cloned())
        .expect("list-windows response must include a windows array");
    assert!(windows.iter().any(|window| {
        window
            .get("title")
            .and_then(|value| value.as_str())
            .map(|title| title == "deskctl list-windows test")
            .unwrap_or(false)
    }));

    let after = deskctl_tmp_screenshot_count();
    assert_eq!(
        before, after,
        "list-windows should not create screenshot artifacts"
    );

    Ok(())
}

#[test]
fn daemon_start_recovers_from_stale_socket() -> Result<()> {
    let _guard = env_lock().lock().unwrap();
    let Some(_env) = SessionEnvGuard::prepare() else {
        eprintln!("Skipping X11 integration test because DISPLAY is not set");
        return Ok(());
    };

    let _window = FixtureWindow::create("deskctl daemon recovery test", "DeskctlDaemon")?;
    let session = TestSession::new("daemon-recovery")?;
    session.create_stale_socket()?;

    session.start_daemon_cli()?;
    let response = successful_json_response(session.run_cli(["--json", "list-windows"])?)
        .expect("list-windows should return valid JSON");

    let windows = response
        .get("data")
        .and_then(|data| data.get("windows"))
        .and_then(|value| value.as_array())
        .expect("CLI JSON response must include windows");
    assert!(windows.iter().any(|window| {
        window
            .get("title")
            .and_then(|value| value.as_str())
            .map(|title| title == "deskctl daemon recovery test")
            .unwrap_or(false)
    }));

    Ok(())
}

#[test]
fn wait_window_returns_matched_window_payload() -> Result<()> {
    let _guard = env_lock().lock().unwrap();
    let Some(_env) = SessionEnvGuard::prepare() else {
        eprintln!("Skipping X11 integration test because DISPLAY is not set");
        return Ok(());
    };

    let title = "deskctl wait window test";
    let _window = FixtureWindow::create(title, "DeskctlWait")?;
    let session = TestSession::new("wait-window-success")?;
    let response = successful_json_response(session.run_cli([
        "--json",
        "wait",
        "window",
        "--selector",
        &format!("title={title}"),
        "--timeout",
        "1",
        "--poll-ms",
        "50",
    ])?)?;

    let window = response
        .get("data")
        .and_then(|data| data.get("window"))
        .expect("wait window should return a matched window");
    assert_eq!(
        window.get("title").and_then(|value| value.as_str()),
        Some(title)
    );
    assert_eq!(
        response
            .get("data")
            .and_then(|data| data.get("wait"))
            .and_then(|value| value.as_str()),
        Some("window")
    );

    Ok(())
}

#[test]
fn ambiguous_fuzzy_selector_returns_candidates() -> Result<()> {
    let _guard = env_lock().lock().unwrap();
    let Some(_env) = SessionEnvGuard::prepare() else {
        eprintln!("Skipping X11 integration test because DISPLAY is not set");
        return Ok(());
    };

    let _window_one = FixtureWindow::create("deskctl ambiguity alpha", "DeskctlAmbiguous")?;
    let _window_two = FixtureWindow::create("deskctl ambiguity beta", "DeskctlAmbiguous")?;
    let session = TestSession::new("selector-ambiguity")?;
    let output = session.run_cli(["--json", "focus", "ambiguity"])?;
    let response = json_response(&output)?;

    assert!(!output.status.success());
    assert_eq!(
        response.get("success").and_then(|value| value.as_bool()),
        Some(false)
    );
    assert_eq!(
        response
            .get("data")
            .and_then(|data| data.get("kind"))
            .and_then(|value| value.as_str()),
        Some("selector_ambiguous")
    );
    assert!(response
        .get("data")
        .and_then(|data| data.get("candidates"))
        .and_then(|value| value.as_array())
        .map(|candidates| candidates.len() >= 2)
        .unwrap_or(false));

    Ok(())
}

#[test]
fn wait_focus_timeout_is_structured() -> Result<()> {
    let _guard = env_lock().lock().unwrap();
    let Some(_env) = SessionEnvGuard::prepare() else {
        eprintln!("Skipping X11 integration test because DISPLAY is not set");
        return Ok(());
    };

    let session = TestSession::new("wait-focus-timeout")?;
    let output = session.run_cli([
        "--json",
        "wait",
        "focus",
        "--selector",
        "title=missing-window-for-wait-focus",
        "--timeout",
        "1",
        "--poll-ms",
        "50",
    ])?;
    let response = json_response(&output)?;

    assert!(!output.status.success());
    assert_eq!(
        response
            .get("data")
            .and_then(|data| data.get("kind"))
            .and_then(|value| value.as_str()),
        Some("timeout")
    );
    assert_eq!(
        response
            .get("data")
            .and_then(|data| data.get("last_observation"))
            .and_then(|value| value.get("kind"))
            .and_then(|value| value.as_str()),
        Some("selector_not_found")
    );

    Ok(())
}
