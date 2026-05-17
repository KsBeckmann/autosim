use std::ops::Range;

use ariadne::{Label, Report, Source};

#[derive(Debug, Clone)]
pub struct SemaError {
    pub message: String,
    pub primary: (Range<usize>, String),
    pub secondary: Vec<(Range<usize>, String)>,
}

impl SemaError {
    pub fn new(message: impl Into<String>, span: Range<usize>, label: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            primary: (span, label.into()),
            secondary: Vec::new(),
        }
    }

    pub fn with_note(mut self, span: Range<usize>, label: impl Into<String>) -> Self {
        self.secondary.push((span, label.into()));
        self
    }

    pub fn report(&self, file_path: &str, source: &str) {
        let (span, label) = &self.primary;
        let mut builder = Report::build(ariadne::ReportKind::Error, (file_path, span.clone()))
            .with_message(&self.message)
            .with_label(
                Label::new((file_path, span.clone()))
                    .with_message(label)
                    .with_color(ariadne::Color::Red),
            );

        for (s, l) in &self.secondary {
            builder = builder.with_label(
                Label::new((file_path, s.clone()))
                    .with_message(l)
                    .with_color(ariadne::Color::Yellow),
            );
        }

        builder
            .finish()
            .eprint((file_path, Source::from(source)))
            .unwrap();
    }
}

pub(crate) fn to_char_span(tok_span: &Range<usize>, token_spans: &[Range<usize>]) -> Range<usize> {
    let start_tok = tok_span.start;
    let end_tok = tok_span.end;
    if start_tok < token_spans.len() {
        let start = token_spans[start_tok].start;
        let end = if end_tok > start_tok && end_tok <= token_spans.len() {
            token_spans[end_tok - 1].end
        } else {
            start
        };
        start..end
    } else {
        let eof = token_spans.last().map(|r| r.end).unwrap_or(0);
        eof..eof
    }
}
