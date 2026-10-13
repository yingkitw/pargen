use anyhow::Result;
use clap::{Parser, Subcommand};
use pargen::Diagnostic;
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::{info, debug, warn};

#[derive(Parser)]
#[command(name = "pargen")]
#[command(about = "A parser generator using ANTLR grammar syntax")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        #[arg(short, long, help = "Path to the .g4 grammar file")]
        grammar: PathBuf,

        #[arg(short, long, help = "Target language (rust, go, typescript, python, java, c, cpp, treesitter)")]
        lang: String,

        #[arg(short, long, help = "Output directory", default_value = ".")]
        output: PathBuf,
    },
    Init {
        #[arg(short, long, help = "Name of the grammar")]
        name: String,

        #[arg(short, long, help = "Output directory", default_value = ".")]
        output: PathBuf,
    },
    Parse {
        #[arg(short, long, help = "Path to the .g4 grammar file")]
        grammar: PathBuf,

        #[arg(short, long, help = "Input text to parse")]
        input: String,

        #[arg(short, long, help = "Output format (json, tree)", default_value = "tree")]
        format: String,
    },
    Check {
        #[arg(short, long, help = "Path to the .g4 grammar file")]
        grammar: PathBuf,
    },
}

fn main() -> ExitCode {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match run(cli) {
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode> {
    match cli.command {
        Commands::Generate {
            grammar,
            lang,
            output,
        } => {
            let grammar_path = grammar.to_string_lossy().to_string();
            let source = std::fs::read_to_string(&grammar_path)?;
            let (grammar_ast, diagnostics) = pargen::parse_grammar_source_detailed(&source);
            let had_errors = print_diagnostics(&diagnostics);
            let grammar_ast = match grammar_ast {
                Some(g) => g,
                None => {
                    anyhow::bail!("Grammar could not be recovered; see diagnostics above");
                }
            };
            let output_str = output.to_string_lossy().to_string();
            pargen::generate(grammar_ast, &lang, &output_str)?;
            info!("Generated {} parser in {}", lang, output_str);
            Ok(exit_code_for(had_errors))
        }
        Commands::Init { name, output } => {
            let grammar_content = format!(
r#"grammar {name};

// Lexer rules
WS: [ \t\r\n]+ -> skip;

// Parser rules
start: EOF;
"#
            );
            let output_dir = output;
            std::fs::create_dir_all(&output_dir)?;
            let grammar_file = output_dir.join(format!("{}.g4", name));
            std::fs::write(&grammar_file, &grammar_content)?;
            info!("Created grammar file: {}", grammar_file.display());
            Ok(ExitCode::from(0))
        }
        Commands::Parse {
            grammar,
            input,
            format,
        } => {
            let grammar_path = grammar.to_string_lossy().to_string();
            let grammar_ast = pargen::parse_grammar_file(&grammar_path)?;
            info!("Grammar: {} ({} rules)", grammar_ast.name, grammar_ast.rules.len());
            debug!("Input: {}", input);
            debug!("Format: {}", format);
            info!("Note: Parsing requires the generated parser. Use 'pargen generate' first.");
            Ok(ExitCode::from(0))
        }
        Commands::Check { grammar } => {
            let grammar_path = grammar.to_string_lossy().to_string();
            let source = std::fs::read_to_string(&grammar_path)?;
            let (grammar_ast, diagnostics) = pargen::parse_grammar_source_detailed(&source);
            let had_errors = print_diagnostics(&diagnostics);
            let grammar_ast = match grammar_ast {
                Some(g) => g,
                None => {
                    anyhow::bail!("Grammar header could not be parsed; aborting");
                }
            };
            info!("Grammar '{}' parsed (with possible recovery)", grammar_ast.name);
            info!("  Lexer rules: {}", grammar_ast.lexer_rules().len());
            info!("  Parser rules: {}", grammar_ast.parser_rules().len());
            let preds = grammar_ast.predicates();
            if preds.is_empty() {
                info!("  Semantic predicates: 0");
            } else {
                info!("  Semantic predicates: {} (not enforced by codegen yet)", preds.len());
                for p in &preds {
                    info!(
                        "    - {}[alt {}].{} = {{ {} }}?",
                        p.rule_name, p.alternative_index, p.element_index, p.source
                    );
                }
            }
            let channels = grammar_ast.channels();
            if channels.is_empty() {
                info!("  Lexer channels: 0");
            } else {
                info!("  Lexer channels: {} (routing not emitted by codegen yet)", channels.len());
                for ch in &channels {
                    if ch.assigned_rules.is_empty() {
                        info!("    - {} (no rules)", ch.name);
                    } else {
                        info!("    - {} ← {}", ch.name, ch.assigned_rules.join(", "));
                    }
                }
            }
            for rule in &grammar_ast.rules {
                let kind = if rule.is_lexer_rule() { "lexer" } else { "parser" };
                let skip = if rule.is_skip() { " (skip)" } else { "" };
                debug!("  [{}] {}{} ({} alternatives)", kind, rule.name, skip, rule.alternatives.len());
            }
            Ok(exit_code_for(had_errors))
        }
    }
}

fn print_diagnostics(diagnostics: &[Diagnostic]) -> bool {
    let mut had_errors = false;
    for d in diagnostics {
        if matches!(d.severity, pargen::DiagnosticSeverity::Error) {
            had_errors = true;
        }
        match &d.location {
            Some(loc) => eprintln!("{}: {}: {}", d.severity, loc, d.message),
            None => eprintln!("{}: {}", d.severity, d.message),
        }
        if let Some(note) = &d.note {
            eprintln!("  = note: {}", note);
        }
    }
    if had_errors {
        warn!("Grammar has errors; generation may fail");
    }
    had_errors
}

fn exit_code_for(had_errors: bool) -> ExitCode {
    if had_errors {
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}
