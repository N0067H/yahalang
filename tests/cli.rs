use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);

fn temp_source(contents: &str) -> std::path::PathBuf {
    let sequence = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "yahalang-cli-test-{}-{sequence}.yaha",
        std::process::id()
    ));
    fs::write(&path, contents).expect("write temporary source file");
    path
}

#[test]
fn run_subcommand_executes_a_source_file_with_selected_language() {
    let source = temp_source("우이 우이 야하 이");
    let mut child = Command::new(env!("CARGO_BIN_EXE_yahalang"))
        .arg("run")
        .arg(&source)
        .args(["--lang", "ko"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start yahalang run");
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(b"20\n22\n")
        .expect("write program input");

    let output = child.wait_with_output().expect("wait for yahalang run");
    fs::remove_file(source).expect("remove temporary source file");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"42");
    assert!(output.stderr.is_empty());
}

#[test]
fn repl_runs_each_entered_line_and_separates_its_output_from_the_prompt() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_yahalang"))
        .args(["repl", "--lang", "en"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start yahalang repl");
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(b"ura yahayaha i\n:q\n")
        .expect("write REPL commands");

    let output = child.wait_with_output().expect("wait for yahalang repl");

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "yaha> 2\nyaha> ");
    assert!(output.stderr.is_empty());
}

#[test]
fn errors_keep_source_locations_and_use_the_playful_voice() {
    let source = temp_source(".");
    let output = Command::new(env!("CARGO_BIN_EXE_yahalang"))
        .arg("run")
        .arg(&source)
        .output()
        .expect("start yahalang with invalid source");
    fs::remove_file(source).expect("remove temporary source file");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("1:1: haa?"), "{stderr}");
}

#[test]
fn repl_can_show_friendly_runtime_error_details() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_yahalang"))
        .args(["repl", "--lang", "en", "--error-style", "friendly"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start yahalang repl with friendly errors");
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(b"yaha\n:q\n")
        .expect("write REPL commands");

    let output = child.wait_with_output().expect("wait for yahalang repl");

    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("runtime error at line 1, instruction 1: stack underflow")
    );
}

#[test]
fn set_language_persists_in_config_and_cli_option_overrides_it() {
    let sequence = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let config_dir = std::env::temp_dir().join(format!(
        "yahalang-config-test-{}-{sequence}",
        std::process::id()
    ));
    let set = Command::new(env!("CARGO_BIN_EXE_yahalang"))
        .args(["set", "lang", "ja"])
        .env("YAHALANG_CONFIG_DIR", &config_dir)
        .output()
        .expect("set configured language");
    assert!(set.status.success());
    assert_eq!(
        fs::read_to_string(config_dir.join("yahalang/config.toml")).unwrap(),
        "lang = \"ja\"\n"
    );

    let japanese_source = temp_source("ウイ ウイ ヤハ イ");
    let mut run = Command::new(env!("CARGO_BIN_EXE_yahalang"))
        .arg("run")
        .arg(&japanese_source)
        .env("YAHALANG_CONFIG_DIR", &config_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("run using configured Japanese language");
    run.stdin.take().unwrap().write_all(b"20\n22\n").unwrap();
    let japanese_output = run.wait_with_output().unwrap();
    assert!(japanese_output.status.success());
    assert_eq!(japanese_output.stdout, b"42");

    let english_source = temp_source("ura yaha i");
    let override_output = Command::new(env!("CARGO_BIN_EXE_yahalang"))
        .arg("run")
        .arg(&english_source)
        .args(["--lang", "en"])
        .env("YAHALANG_CONFIG_DIR", &config_dir)
        .output()
        .expect("run with explicit language override");
    assert!(override_output.status.success());
    assert_eq!(override_output.stdout, b"1");

    fs::remove_file(japanese_source).unwrap();
    fs::remove_file(english_source).unwrap();
    fs::remove_dir_all(config_dir).unwrap();
}
