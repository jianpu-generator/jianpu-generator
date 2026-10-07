use clap::{Parser, Subcommand};
use jianpu_generator::cli::generate::{
    generate_midi, generate_mp3, generate_pdf, generate_svg, generate_wav,
};
use jianpu_generator::cli::{check, GenerateInput};
use jianpu_generator::{self as jg, error_reporter};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "jianpu", about = "Generate JianPu notation files")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        #[command(subcommand)]
        format: GenerateFormat,
    },
    Check {
        input: PathBuf,
    },
    /// Rewrite positional lyric lines into `lyrics[...]` parts. Rewrites the
    /// given files in place; with no files, filters stdin to stdout.
    MigrateLyrics {
        paths: Vec<PathBuf>,
    },
}

#[derive(Subcommand)]
enum GenerateFormat {
    Pdf {
        input: PathBuf,
        #[arg(long, help = "Output file stem (extension is added automatically)")]
        output: Option<PathBuf>,
        #[arg(long, value_delimiter = ',', num_args = 0.., help = "Comma-separated list of track names to include (e.g. --tracks S1,S2)")]
        tracks: Vec<String>,
        #[arg(
            long,
            help = "Generate one file per track instead of a single combined file"
        )]
        split_tracks: bool,
    },
    Svg {
        input: PathBuf,
        #[arg(long, help = "Output file stem (extension is added automatically)")]
        output: Option<PathBuf>,
        #[arg(long, value_delimiter = ',', num_args = 0.., help = "Comma-separated list of track names to include (e.g. --tracks S1,S2)")]
        tracks: Vec<String>,
        #[arg(
            long,
            help = "Generate one file per track instead of a single combined file"
        )]
        split_tracks: bool,
    },
    Midi {
        input: PathBuf,
        #[arg(long, help = "Output file stem (extension is added automatically)")]
        output: Option<PathBuf>,
        #[arg(long, value_delimiter = ',', num_args = 0.., help = "Comma-separated list of track names to include (e.g. --tracks S1,S2)")]
        tracks: Vec<String>,
        #[arg(
            long,
            help = "Generate one file per track instead of a single combined file"
        )]
        split_tracks: bool,
    },
    Wav {
        input: PathBuf,
        #[arg(long, help = "Output file stem (extension is added automatically)")]
        output: Option<PathBuf>,
        #[arg(long, value_delimiter = ',', num_args = 0.., help = "Comma-separated list of track names to include (e.g. --tracks S1,S2)")]
        tracks: Vec<String>,
        #[arg(
            long,
            help = "Generate one file per track instead of a single combined file"
        )]
        split_tracks: bool,
    },
    Mp3 {
        input: PathBuf,
        #[arg(long, help = "Output file stem (extension is added automatically)")]
        output: Option<PathBuf>,
        #[arg(long, value_delimiter = ',', num_args = 0.., help = "Comma-separated list of track names to include (e.g. --tracks S1,S2)")]
        tracks: Vec<String>,
        #[arg(
            long,
            help = "Generate one file per track instead of a single combined file"
        )]
        split_tracks: bool,
    },
}

fn main() -> ExitCode {
    let args = Args::parse();

    let result = match args.command {
        Commands::Generate { format } => run_generate(format).map(|()| true),
        Commands::Check { input } => run_check(&input),
        Commands::MigrateLyrics { paths } => run_migrate_lyrics(&paths).map(|()| true),
    };

    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            error_reporter::render(&e);
            ExitCode::FAILURE
        }
    }
}

fn run_migrate_lyrics(paths: &[PathBuf]) -> Result<(), jg::error::IrrecoverableError> {
    if paths.is_empty() {
        let mut source = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut source).map_err(|error| {
            jg::error::IrrecoverableError::new(jg::error::IrrecoverableErrorKind::IoReadFailed {
                span: jg::error::Span::new(0, 0),
                path: PathBuf::from("<stdin>"),
                source: error.to_string(),
            })
        })?;
        print!("{}", jg::migrate_lyrics::migrate_lyrics_syntax(&source));
        return Ok(());
    }
    paths.iter().try_for_each(|path| {
        let source = jianpu_generator::cli::read_source(path)?;
        let migrated = jg::migrate_lyrics::migrate_lyrics_syntax(&source);
        jianpu_generator::cli::write_file(path, migrated.as_bytes())
    })
}

/// Returns `Ok(true)` when the file parses with no errors, `Ok(false)` when it
/// parses but has recoverable errors.
fn run_check(input: &std::path::Path) -> Result<bool, jg::error::IrrecoverableError> {
    let outcome = check(input)?;
    for diagnostic in &outcome.diagnostics {
        eprintln!("{}: {}", input.display(), diagnostic.message());
    }
    if outcome.ok {
        println!("{input:?}: ok");
    }
    Ok(outcome.ok)
}

fn run_generate(format: GenerateFormat) -> Result<(), jg::error::IrrecoverableError> {
    match format {
        GenerateFormat::Pdf {
            input,
            output,
            tracks,
            split_tracks,
        } => generate_pdf(&GenerateInput {
            input,
            output,
            tracks,
            split_tracks,
        }),
        GenerateFormat::Svg {
            input,
            output,
            tracks,
            split_tracks,
        } => generate_svg(&GenerateInput {
            input,
            output,
            tracks,
            split_tracks,
        }),
        GenerateFormat::Midi {
            input,
            output,
            tracks,
            split_tracks,
        } => generate_midi(&GenerateInput {
            input,
            output,
            tracks,
            split_tracks,
        }),
        GenerateFormat::Wav {
            input,
            output,
            tracks,
            split_tracks,
        } => generate_wav(&GenerateInput {
            input,
            output,
            tracks,
            split_tracks,
        }),
        GenerateFormat::Mp3 {
            input,
            output,
            tracks,
            split_tracks,
        } => generate_mp3(&GenerateInput {
            input,
            output,
            tracks,
            split_tracks,
        }),
    }
}
