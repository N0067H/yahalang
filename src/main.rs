use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use yahalang::executor::execute;
use yahalang::language::Language;
use yahalang::lexer::lex;
use yahalang::parser::parse;

#[derive(Debug, Parser)]
#[command(name = "yahalang", version, about = "Run Yahalang programs")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Start an interactive one-line REPL")]
    Repl {
        #[arg(long, value_enum)]
        lang: Option<CliLanguage>,
        #[arg(long, value_enum, default_value = "usagi")]
        error_style: ErrorStyle,
    },
    #[command(about = "Run a program from a source file")]
    Run {
        file: PathBuf,
        #[arg(long, value_enum)]
        lang: Option<CliLanguage>,
        #[arg(long, value_enum, default_value = "usagi")]
        error_style: ErrorStyle,
    },
    #[command(about = "Save a default setting")]
    Set {
        key: String,
        #[arg(value_enum)]
        language: CliLanguage,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliLanguage {
    En,
    Ko,
    Ja,
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum ErrorStyle {
    #[default]
    Usagi,
    Friendly,
}

impl CliLanguage {
    fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ko => "ko",
            Self::Ja => "ja",
        }
    }
}

impl From<CliLanguage> for Language {
    fn from(language: CliLanguage) -> Self {
        match language {
            CliLanguage::En => Self::English,
            CliLanguage::Ko => Self::Korean,
            CliLanguage::Ja => Self::Japanese,
        }
    }
}

fn main() -> ExitCode {
    match dispatch(Cli::parse().command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn dispatch(command: Command) -> Result<(), String> {
    match command {
        Command::Repl { lang, error_style } => selected_language(lang)
            .and_then(|language| repl(language, error_style).map_err(|error| error.to_string()))
            .map_err(|error| format!("yahalang: {error}")),
        Command::Run {
            file,
            lang,
            error_style,
        } => selected_language(lang)
            .and_then(|language| run_file(&file, language, error_style))
            .map_err(|error| format!("{}: {error}", file.display())),
        Command::Set { key, language } => {
            save_setting(&key, language).map_err(|error| format!("yahalang: {error}"))
        }
    }
}

fn save_setting(key: &str, language: CliLanguage) -> Result<(), String> {
    let path = set_config(key, language)?;
    println!("saved {key} = {} to {}", language.code(), path.display());

    Ok(())
}

fn selected_language(override_language: Option<CliLanguage>) -> Result<Language, String> {
    let language = match override_language {
        Some(language) => Some(language),
        None => read_config_language()?,
    }
    .unwrap_or(CliLanguage::En);

    Ok(language.into())
}

fn read_config_language() -> Result<Option<CliLanguage>, String> {
    let path = config_file()?;
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("could not read {}: {error}", path.display())),
    };

    let config = contents
        .parse::<toml::Table>()
        .map_err(|error| format!("invalid config at {}: {error}", path.display()))?;
    if let Some(key) = config.keys().find(|key| key.as_str() != "lang") {
        return Err(format!("unknown setting `{key}` at {}", path.display()));
    }

    let Some(value) = config.get("lang") else {
        return Ok(None);
    };

    let value = value
        .as_str()
        .ok_or_else(|| format!("expected a language string at {}", path.display()))?;

    parse_language(value).map(Some).ok_or_else(|| {
        format!(
            "unsupported language `{value}` at {}; expected en, ko, or ja",
            path.display()
        )
    })
}

fn set_config(key: &str, language: CliLanguage) -> Result<PathBuf, String> {
    if key != "lang" {
        return Err(format!("unknown setting `{key}`; available setting: lang"));
    }

    let path = config_file()?;
    let parent = path
        .parent()
        .ok_or_else(|| "config path has no parent directory".to_owned())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
    fs::write(&path, format!("lang = \"{}\"\n", language.code()))
        .map_err(|error| format!("could not write {}: {error}", path.display()))?;

    Ok(path)
}

fn parse_language(value: &str) -> Option<CliLanguage> {
    match value {
        "en" => Some(CliLanguage::En),
        "ko" => Some(CliLanguage::Ko),
        "ja" => Some(CliLanguage::Ja),
        _ => None,
    }
}

fn config_file() -> Result<PathBuf, String> {
    let config_root = if let Some(path) = std::env::var_os("YAHALANG_CONFIG_DIR") {
        PathBuf::from(path)
    } else {
        default_config_dir()?
    };

    Ok(config_root.join("yahalang").join("config.toml"))
}

fn default_config_dir() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        return std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .ok_or_else(|| "APPDATA is not set".to_owned());
    }

    #[cfg(target_os = "macos")]
    {
        return std::env::var_os("HOME")
            .map(|home| PathBuf::from(home).join("Library/Application Support"))
            .ok_or_else(|| "HOME is not set".to_owned());
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .ok_or_else(|| "XDG_CONFIG_HOME and HOME are not set".to_owned())
    }
}

fn run_file(file: &PathBuf, language: Language, error_style: ErrorStyle) -> Result<(), String> {
    let source = fs::read_to_string(file).map_err(|error| error.to_string())?;
    let program = compile(&source, language, error_style)?;
    let stdin = io::stdin();
    let stdout = io::stdout();

    execute(&program, &mut stdin.lock(), &mut stdout.lock())
        .map_err(|error| format_runtime_error(error, error_style))
}

fn repl(language: Language, error_style: ErrorStyle) -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();
    let mut line = String::new();

    loop {
        write!(output, "yaha> ")?;
        output.flush()?;

        line.clear();
        if input.read_line(&mut line)? == 0 {
            break;
        }

        let command = line.trim_end_matches(['\r', '\n']);
        match command {
            ":quit" | ":q" => break,
            "" => {}
            _ => run_repl_source(command, language, error_style, &mut input, &mut output),
        }
    }

    Ok(())
}

fn run_repl_source<R: BufRead, W: Write>(
    source: &str,
    language: Language,
    error_style: ErrorStyle,
    input: &mut R,
    output: &mut W,
) {
    let program = match compile(source, language, error_style) {
        Ok(program) => program,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };

    let mut repl_output = ReplOutput::new(output);
    let result = execute(&program, input, &mut repl_output);

    if repl_output
        .last_byte
        .is_some_and(|last_byte| last_byte != b'\n')
    {
        let _ = writeln!(repl_output);
        let _ = repl_output.flush();
    }

    if let Err(error) = result {
        eprintln!("{}", format_runtime_error(error, error_style));
    }
}

struct ReplOutput<'a, W> {
    output: &'a mut W,
    last_byte: Option<u8>,
}

impl<'a, W> ReplOutput<'a, W> {
    fn new(output: &'a mut W) -> Self {
        Self {
            output,
            last_byte: None,
        }
    }
}

impl<W: Write> Write for ReplOutput<'_, W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let written = self.output.write(buffer)?;
        if written > 0 {
            self.last_byte = Some(buffer[written - 1]);
        }

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

fn compile(
    source: &str,
    language: Language,
    error_style: ErrorStyle,
) -> Result<yahalang::parser::Program, String> {
    let tokens = lex(source, language).map_err(|error| {
        let (line, column) = location(source, error.span.start);
        match error_style {
            ErrorStyle::Usagi => format!("{line}:{column}: {error}"),
            ErrorStyle::Friendly => format!(
                "{line}:{column}: unexpected character {:?}",
                error.character
            ),
        }
    })?;

    parse(&tokens).map_err(|error| {
        let (line, column) = location(source, error.span.start);
        match error_style {
            ErrorStyle::Usagi => format!("{line}:{column}: {error}"),
            ErrorStyle::Friendly => format!("{line}:{column}: {}", error.message),
        }
    })
}

fn format_runtime_error(error: yahalang::executor::RuntimeError, style: ErrorStyle) -> String {
    match style {
        ErrorStyle::Usagi => error.to_string(),
        ErrorStyle::Friendly => format!(
            "runtime error at line {}, instruction {}: {}",
            error.line, error.instruction, error.message
        ),
    }
}

fn location(source: &str, byte_offset: usize) -> (usize, usize) {
    let prefix = &source[..byte_offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let column = source[line_start..byte_offset].chars().count() + 1;

    (line, column)
}
