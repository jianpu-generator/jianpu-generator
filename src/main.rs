use clap::{Parser, Subcommand};
use jianpu_generator::{self as jg, error_reporter};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod generate;

#[cfg(feature = "wav")]
pub(crate) static SF2_BYTES: &[u8] = include_bytes!("../fonts/GeneralUser_GS.sf2");

#[cfg(feature = "pdf")]
static SANS_SERIF_SC_FONT: &[u8] = include_bytes!("../fonts/SourceHanSansSC-Regular.otf");
#[cfg(feature = "pdf")]
static SANS_SERIF_TC_FONT: &[u8] = include_bytes!("../fonts/SourceHanSansTC-Regular.otf");
#[cfg(feature = "pdf")]
static MONOSPACE_FONT: &[u8] = include_bytes!("../fonts/NotoSansMono-Regular.ttf");

#[cfg(feature = "pdf")]
pub(crate) fn default_pdf_fonts() -> jg::pdf::PdfFonts {
    jg::pdf::PdfFonts {
        sans_serif_sc: SANS_SERIF_SC_FONT.to_vec(),
        sans_serif_tc: SANS_SERIF_TC_FONT.to_vec(),
        monospace: MONOSPACE_FONT.to_vec(),
    }
}

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
}

fn main() -> ExitCode {
    let args = Args::parse();

    let result = match args.command {
        Commands::Generate { format } => generate::run_generate(format).map(|()| true),
        Commands::Check { input } => run_check(&input),
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

/// Returns `Ok(true)` when the file parses with no errors, `Ok(false)` when it
/// parses but has recoverable errors.
fn run_check(input: &Path) -> Result<bool, jg::error::IrrecoverableError> {
    let score = parse_and_group(input)?;
    let diagnostics = jg::collect_measure_diagnostics(&score);

    for diagnostic in &diagnostics {
        eprintln!("{}: {}", input.display(), diagnostic.message());
    }

    let has_errors = diagnostics
        .iter()
        .any(|d| matches!(d, jg::error::Diagnostic::Error(_)));
    if has_errors {
        return Ok(false);
    }

    println!("{input:?}: ok");
    Ok(true)
}

fn read_source(input: &Path) -> Result<String, jg::error::IrrecoverableError> {
    std::fs::read_to_string(input).map_err(|e| {
        jg::error::IrrecoverableError::new(jg::error::IrrecoverableErrorKind::IoReadFailed {
            span: jg::error::Span::new(0, 0),
            path: input.to_path_buf(),
            source: e.to_string(),
        })
    })
}

fn parse_and_group(input: &Path) -> Result<jg::ast::grouped::Score, jg::error::IrrecoverableError> {
    let content = read_source(input)?;
    let filename = input.to_string_lossy().to_string();
    let doc = jg::parser::parse(&content, &filename, &[]).map_err(|e| e.with_path(input))?;
    jg::grouper::group(doc).map_err(|e| e.with_path(input))
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), jg::error::IrrecoverableError> {
    std::fs::write(path, bytes).map_err(|e| {
        jg::error::IrrecoverableError::new(jg::error::IrrecoverableErrorKind::IoWriteFailed {
            span: jg::error::Span::new(0, 0),
            path: path.to_path_buf(),
            source: e.to_string(),
        })
    })
}
