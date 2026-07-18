use std::process::{Command, Output};

use serde_json::Value;

fn oanda() -> Command {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("oanda"));
    for name in [
        "OANDA_ACCESS_TOKEN",
        "OANDA_TOKEN",
        "OANDA_ACCOUNT_ID",
        "OANDA_ENVIRONMENT",
        "OANDA_DATETIME_FORMAT",
        "OANDA_REQUEST_TIMEOUT_SECS",
        "OANDA_CONNECT_TIMEOUT_SECS",
    ] {
        command.env_remove(name);
    }
    command
}

fn json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap_or_else(|error| {
        panic!(
            "expected JSON output ({error}): {}",
            String::from_utf8_lossy(bytes)
        )
    })
}

fn run(command: &mut Command) -> Output {
    command.output().expect("oanda command should start")
}

#[test]
fn version_is_available_without_configuration() {
    let output = run(oanda().arg("--version"));
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        concat!("oanda ", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn legacy_token_variable_is_not_accepted() {
    let output = run(oanda()
        .env("OANDA_TOKEN", "legacy-token")
        .args(["account", "list"]));
    assert_eq!(output.status.code(), Some(3));
    let error = json(&output.stderr);
    assert_eq!(error["error"]["kind"], "configuration");
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("OANDA_ACCESS_TOKEN")
    );
}

#[test]
fn canonical_access_token_variable_is_accepted() {
    let output = run(oanda()
        .env("OANDA_ACCESS_TOKEN", "test-access-token")
        .args(["account", "get"]));
    assert_eq!(output.status.code(), Some(3));
    let error = json(&output.stderr);
    assert_eq!(error["error"]["kind"], "configuration");
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Account ID required")
    );
}

#[test]
fn environment_variable_selects_practice_for_a_typed_dry_run() {
    let output = run(oanda()
        .env("OANDA_ENVIRONMENT", "practice")
        .env("OANDA_ACCOUNT_ID", "101-001-123-001")
        .args([
            "--dry-run",
            "order",
            "market",
            "--instrument",
            "EUR_USD",
            "--units",
            "-100",
            "--position-fill",
            "DEFAULT",
        ]));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan = json(&output.stdout);
    assert_eq!(plan["environment"], "practice");
    assert_eq!(plan["method"], "POST");
    assert_eq!(plan["body"]["order"]["units"], "-100");
}

#[test]
fn mutations_default_to_practice() {
    let output = run(oanda().args([
        "--account-id",
        "101-001-123-001",
        "--dry-run",
        "order",
        "market",
        "--instrument",
        "EUR_USD",
        "--units",
        "100",
    ]));
    assert!(output.status.success());
    assert_eq!(json(&output.stdout)["environment"], "practice");
}

#[test]
fn live_dry_runs_need_no_extra_confirmation() {
    let args = [
        "--environment",
        "live",
        "--account-id",
        "101-001-123-001",
        "--dry-run",
        "order",
        "cancel",
        "123",
    ];
    let output = run(oanda().args(args));
    assert!(output.status.success());
    assert_eq!(json(&output.stdout)["environment"], "live");
}

#[test]
fn removed_confirm_live_flag_is_rejected() {
    let output = run(oanda().arg("--confirm-live").args(["account", "list"]));
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(json(&output.stderr)["error"]["kind"], "validation");
}

#[test]
fn schema_is_machine_readable_without_credentials() {
    let output = run(oanda().args(["schema", "--json"]));
    assert!(output.status.success());
    let schema = json(&output.stdout);
    assert_eq!(schema["schemaVersion"], 2);
    assert!(schema["configuration"]["liveMutationConfirmation"].is_null());
    assert!(
        schema["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| { command["path"] == "order market" && command["mutation"] == true })
    );
}

#[test]
fn usage_errors_are_structured_json() {
    let output = run(oanda().args(["--environment", "demo", "account", "list"]));
    assert_eq!(output.status.code(), Some(2));
    let error = json(&output.stderr);
    assert_eq!(error["error"]["kind"], "validation");
    assert_eq!(error["error"]["exitCode"], 2);
}

#[test]
fn dry_run_redacts_secret_fields_in_raw_json() {
    let output = run(oanda().args([
        "--environment",
        "practice",
        "--account-id",
        "101-001-123-001",
        "--dry-run",
        "account",
        "configure",
        "--body",
        "{\"alias\":\"Primary\",\"apiToken\":\"do-not-print\"}",
    ]));
    assert!(output.status.success());
    let plan = json(&output.stdout);
    assert_eq!(plan["body"]["apiToken"], "[REDACTED]");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("do-not-print"));
}
