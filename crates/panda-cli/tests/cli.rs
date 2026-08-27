use std::process::Command;

#[test]
fn version_reports_the_workspace_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_panda"))
        .arg("--version")
        .output()
        .expect("panda binary should run");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"2.0.0-alpha.1\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn operational_commands_fail_explicitly_during_m0() {
    let output = Command::new(env!("CARGO_BIN_EXE_panda"))
        .arg("serve")
        .output()
        .expect("panda binary should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("commands are not implemented"));
}
