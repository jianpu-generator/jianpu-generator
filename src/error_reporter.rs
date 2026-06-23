use crate::error::{IrrecoverableError, Span, Warning};
use ariadne::Config;

pub fn render(e: &IrrecoverableError) {
    render_to_writer(e, std::io::stderr(), None, Config::default());
}

/// Render a pretty error using an in-memory source string (for WASM and other non-FS hosts).
///
/// Uses plain text (no ANSI color codes) so the output is safe to display in a web UI.
pub fn render_with_source(source: &str, e: &IrrecoverableError) -> String {
    let mut buf = Vec::new();
    render_to_writer(
        e,
        &mut buf,
        Some(source),
        Config::default().with_color(false),
    );
    String::from_utf8_lossy(&buf).into_owned()
}

/// Render a pretty warning for a [`Warning`] using an in-memory source string.
///
/// Uses plain text (no ANSI color codes) so the output is safe to display in a web UI.
pub fn render_warning_with_source(source: &str, e: &Warning) -> String {
    use ariadne::{Label, Report, ReportKind, Source};

    let filename = "input";
    let char_start = source[..e.span.start.min(source.len())].chars().count();
    let char_end = source[..e.span.end.min(source.len())].chars().count();
    let span = (filename, char_start..char_end);

    let mut buf = Vec::new();
    if Report::build(ReportKind::Warning, span.clone())
        .with_config(Config::default().with_color(false))
        .with_message(&e.message)
        .with_label(Label::new(span).with_message(&e.message))
        .finish()
        .write((filename, Source::from(source)), &mut buf)
        .is_err()
    {
        buf.extend_from_slice(format!("warning: {}", e.message).as_bytes());
    }
    String::from_utf8_lossy(&buf).into_owned()
}

fn render_to_writer(
    e: &IrrecoverableError,
    mut writer: impl std::io::Write,
    source: Option<&str>,
    config: Config,
) {
    use ariadne::{Label, Report, ReportKind, Source};

    let source_text = match source {
        Some(s) => Some(s.to_owned()),
        None => e
            .path
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok()),
    };

    let Some(source_text) = source_text else {
        let message = e.message();
        writeln!(writer, "error: {message}").ok();
        return;
    };

    let filename = e
        .path
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "input".to_owned());
    // ariadne indexes by Unicode character count, not by byte offset.
    let span = e.span().copied().unwrap_or(Span::new(0, 0));
    let char_start = source_text[..span.start.min(source_text.len())]
        .chars()
        .count();
    let char_end = source_text[..span.end.min(source_text.len())]
        .chars()
        .count();
    let span = (filename.clone(), char_start..char_end);
    let message = e.message();

    if Report::build(ReportKind::Error, span.clone())
        .with_config(config)
        .with_message(&message)
        .with_label(Label::new(span).with_message(&message))
        .finish()
        .write((filename, Source::from(source_text.as_str())), &mut writer)
        .is_err()
    {
        writeln!(writer, "error: {message}").ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};
    use std::path::PathBuf;

    fn write_temp_file(name: &str, content: &str) -> PathBuf {
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn render_output_contains_message() {
        let path = write_temp_file("test_render.jianpu", "1 2 3 4\n");
        let e = IrrecoverableError::new(IrrecoverableErrorKind::InternalInvariant {
            span: Span::new(0, 1),
            detail: "something went wrong".to_string(),
        })
        .with_path(&path);

        let mut buf = Vec::new();
        render_to_writer(&e, &mut buf, None, Config::default());
        let output = String::from_utf8_lossy(&buf);
        assert!(
            output.contains("something went wrong"),
            "output was: {output}"
        );
    }

    #[test]
    fn render_with_source_shows_code_block() {
        let source = "1 2 3 4\n";
        let e = IrrecoverableError::new(IrrecoverableErrorKind::InternalInvariant {
            span: Span::new(0, 1),
            detail: "something went wrong".to_string(),
        });

        let output = render_with_source(source, &e);
        assert!(
            output.contains('│'),
            "expected ariadne code block, got: {output}"
        );
        assert!(output.contains("something went wrong"));
    }

    #[test]
    fn render_with_source_has_no_ansi_codes() {
        let source = "1 2 3 4\n";
        let e = IrrecoverableError::new(IrrecoverableErrorKind::InternalInvariant {
            span: Span::new(0, 1),
            detail: "something went wrong".to_string(),
        });

        let output = render_with_source(source, &e);
        assert!(
            !output.contains('\x1b'),
            "web report must not contain ANSI escapes, got: {output}"
        );
    }

    #[test]
    fn render_shows_code_block_when_source_contains_multibyte_unicode() {
        // Each Chinese character is 3 bytes. The error token "x" is at byte offset 12
        // (3 bytes × 4 chars = 12), but at character offset 4.
        // Without the byte→char conversion ariadne would look past end-of-source
        // and silently omit the code block.
        let source = "你好世界 x 4\n";
        let path = write_temp_file("test_unicode_render.jianpu", source);
        let token_byte_start = "你好世界 ".len(); // 3*4 + 1 = 13
        let e = IrrecoverableError::new(IrrecoverableErrorKind::InternalInvariant {
            span: Span::new(token_byte_start, token_byte_start + 1),
            detail: "bad token".to_string(),
        })
        .with_path(&path);

        let mut buf = Vec::new();
        render_to_writer(&e, &mut buf, None, Config::default());
        let output = String::from_utf8_lossy(&buf);
        // The code block must appear — presence of '│' confirms ariadne rendered it.
        assert!(
            output.contains('│'),
            "expected ariadne code block (│) in output, got: {output}"
        );
        assert!(output.contains("bad token"), "output was: {output}");
    }

    #[test]
    fn render_falls_back_when_path_is_none() {
        let e = IrrecoverableError::new(IrrecoverableErrorKind::InternalInvariant {
            span: Span::new(0, 1),
            detail: "some error".to_string(),
        });
        let mut buf = Vec::new();
        render_to_writer(&e, &mut buf, None, Config::default());
        let output = String::from_utf8_lossy(&buf);
        assert!(output.contains("some error"), "output was: {output}");
    }

    #[test]
    fn render_falls_back_when_file_unreadable() {
        let e = IrrecoverableError::new(IrrecoverableErrorKind::InternalInvariant {
            span: Span::new(0, 1),
            detail: "some error".to_string(),
        })
        .with_path("/nonexistent/path.jianpu");
        let mut buf = Vec::new();
        render_to_writer(&e, &mut buf, None, Config::default());
        let output = String::from_utf8_lossy(&buf);
        assert!(output.contains("some error"), "output was: {output}");
    }
}
