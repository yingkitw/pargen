use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, debug};

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

        #[arg(short, long, help = "Target language (rust, go, typescript, python, java, c, cpp)")]
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

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            grammar,
            lang,
            output,
        } => {
            let grammar_path = grammar.to_string_lossy().to_string();
            let grammar_ast = pargen::parse_grammar_file(&grammar_path)?;
            let output_str = output.to_string_lossy().to_string();
            pargen::generate(grammar_ast, &lang, &output_str)?;
            info!("Generated {} parser in {}", lang, output_str);
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
        }
        Commands::Check { grammar } => {
            let grammar_path = grammar.to_string_lossy().to_string();
            let grammar_ast = pargen::parse_grammar_file(&grammar_path)?;
            info!("Grammar '{}' parsed successfully", grammar_ast.name);
            info!("  Lexer rules: {}", grammar_ast.lexer_rules().len());
            info!("  Parser rules: {}", grammar_ast.parser_rules().len());
            for rule in &grammar_ast.rules {
                let kind = if rule.is_lexer_rule() { "lexer" } else { "parser" };
                let skip = if rule.is_skip() { " (skip)" } else { "" };
                debug!("  [{}] {}{} ({} alternatives)", kind, rule.name, skip, rule.alternatives.len());
            }
        }
    }

    Ok(())
}
