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
    let source = temp_source("이이이야하 이이이야하 야하 이이이야하 뿌루루");
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
        .write_all(b"hayaha ppuruppuru yaha iiiiyaha ppururu\n:q\n")
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

    let japanese_source = temp_source("イイイヤハ イイイヤハ ヤハ イイイヤハ プルル");
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

    let english_source = temp_source("hayaha ppuru yaha iiiiyaha ppururu");
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

fn repl_session(source: &str) -> (String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_yahalang"))
        .args(["repl", "--lang", "ko", "--error-style", "friendly"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

#[test]
fn repl_preserves_values_and_array_aliases_between_submissions() {
    let (output, errors) = repl_session(
        "[하야하 야하]\n뿌루\n하야하 야하 하야하 뿌루 야하 =\n:stack\n하야하 야하 @ 이이이야하 뿌루루\n:stack\n:q\n",
    );
    assert!(errors.is_empty(), "{errors}");
    assert!(output.contains("yaha> [[1]]\n"), "{output}");
    assert!(output.contains("yaha> 1\n"), "{output}");
    assert!(output.contains("yaha> []\n"), "{output}");
}

#[test]
fn repl_waits_for_nested_blocks_without_executing_partial_source() {
    let (output, errors) = repl_session(
        "하야하 뿌루 야하\n이야 야하야하\n하야하 뿌루뿌루 야하\n이야 야다야다\n뿌루 이이이야하 뿌루루\n하야하 뿌루 야하 우라 야하\n우라 야다야다\n우라 야하야하\n:stack\n:q\n",
    );
    assert!(errors.is_empty(), "{errors}");
    assert!(output.contains("...> 21\nyaha> [0]\n"), "{output}");
}

#[test]
fn repl_reset_discards_pending_source_and_stack() {
    let (output, errors) = repl_session(
        "하야하 뿌루 야하\n이야 야다야다\n:stack\n:reset\n:stack\n하야하\n뿌루 야하\n:stack\n:q\n",
    );
    assert!(errors.is_empty(), "{errors}");
    assert!(output.contains("...> [1]\n"), "{output}");
    assert!(output.contains("yaha> []\n"), "{output}");
    assert!(output.ends_with("yaha> [1]\nyaha> "), "{output}");
}

#[test]
fn repl_recovers_from_syntax_and_runtime_errors() {
    let (output, errors) =
        repl_session("하야하 뿌루 야하\n하아 뿌루\n야하\n:stack\n:wat\n:reset\n:stack\n:q\n");
    assert!(
        errors.contains("expected yaha, iya, or hat after haa"),
        "{errors}"
    );
    assert!(errors.contains("stack underflow"), "{errors}");
    assert!(errors.contains("unknown REPL command"), "{errors}");
    assert!(output.contains("yaha> [1]\n"), "{output}");
    assert!(output.ends_with("yaha> []\nyaha> "), "{output}");
}

#[test]
fn repl_reports_incomplete_source_at_eof_and_reads_program_input() {
    let (_, errors) = repl_session("이야 야다야다\n");
    assert!(errors.contains("unclosed block"), "{errors}");
    let (output, errors) = repl_session("이이이야하\n42\n:stack\n얏\n:stack\n:q\n");
    assert!(errors.is_empty(), "{errors}");
    assert_eq!(output.matches("[42]").count(), 2, "{output}");
}

#[test]
fn repl_reads_character_input_and_preserves_remaining_input() {
    let (output, errors) = repl_session("우라라\n이이이야하\n가\n:stack\n:q\n");
    assert!(errors.is_empty(), "{errors}");
    assert!(output.contains("[44032]"), "{output}");
}

#[test]
fn repl_array_append_and_pop_keep_state_between_submissions() {
    let (output, errors) = repl_session(
        "[]\n하야하 뿌루 야하\n야다\n뿌루\n:stack\n야다 우라\n:stack\n이이이야하 뿌루루\n야다 우라\n:stack\n:q\n",
    );
    assert!(errors.contains("cannot pop an empty array"), "{errors}");
    assert!(output.contains("[[1]]"), "{output}");
    assert!(output.contains("[[], 1]"), "{output}");
    assert!(output.contains("[[]]"), "{output}");
}

#[test]
fn repl_creates_arrays_with_a_spoken_phrase() {
    let (output, errors) = repl_session("야다\n하야하\n하야하 뿌루 야하\n야다 뿌루\n:stack\n:q\n");
    assert!(errors.is_empty(), "{errors}");
    assert!(output.contains("...> "), "{output}");
    assert!(output.contains("[[1]]"), "{output}");
}

#[test]
fn repl_accepts_split_loop_break_and_continues_after_errors() {
    let (output, errors) = repl_session(
        "우라 이야\n하야하 뿌루 야하\n이야 야다야다\n우라\n이야\n우라 야다야다\n:stack\n:q\n",
    );
    assert!(
        errors.contains("loop break is only valid inside a loop"),
        "{errors}"
    );
    assert!(output.contains("[1]"), "{output}");
}
