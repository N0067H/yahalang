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
        #[arg(long, value_enum, default_value = "en")]
        lang: CliLanguage,
    },
    #[command(about = "Run a program from a source file")]
    Run {
        file: PathBuf,
        #[arg(long, value_enum, default_value = "en")]
        lang: CliLanguage,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliLanguage {
    En,
    Ko,
    Ja,
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
    match Cli::parse().command {
        Command::Repl { lang } => match repl(lang.into()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("yahalang: {error}");
                ExitCode::FAILURE
            }
        },
        Command::Run { file, lang } => match run_file(&file, lang.into()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{}: {error}", file.display());
                ExitCode::FAILURE
            }
        },
    }
}

fn run_file(file: &PathBuf, language: Language) -> Result<(), String> {
    let source = fs::read_to_string(file).map_err(|error| error.to_string())?;
    let program = compile(&source, language)?;
    let stdin = io::stdin();
    let stdout = io::stdout();

    execute(&program, &mut stdin.lock(), &mut stdout.lock()).map_err(|error| error.to_string())
}

fn repl(language: Language) -> io::Result<()> {
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
            _ => run_repl_source(command, language, &mut input, &mut output),
        }
    }

    Ok(())
}

fn run_repl_source<R: BufRead, W: Write>(
    source: &str,
    language: Language,
    input: &mut R,
    output: &mut W,
) {
    match compile(source, language) {
        Ok(program) => {
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
                eprintln!("{error}");
            }
        }
        Err(error) => eprintln!("{error}"),
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

fn compile(source: &str, language: Language) -> Result<yahalang::parser::Program, String> {
    let tokens = lex(source, language).map_err(|error| {
        let (line, column) = location(source, error.span.start);
        format!("{line}:{column}: {error}")
    })?;

    parse(&tokens).map_err(|error| {
        let (line, column) = location(source, error.span.start);
        format!("{line}:{column}: {error}")
    })
}

fn location(source: &str, byte_offset: usize) -> (usize, usize) {
    let prefix = &source[..byte_offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let column = source[line_start..byte_offset].chars().count() + 1;

    (line, column)
}
