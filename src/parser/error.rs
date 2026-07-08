use crate::lexer::Token;
use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::error::{Rich, RichPattern, RichReason};
use std::ops::Range;

#[derive(Debug)]
pub struct ParseError {
    pub span: std::ops::Range<usize>,
    pub message: String,
}

impl ParseError {
    #[must_use]
    pub fn from_rich(err: &Rich<'_, Token>, token_spans: &[Range<usize>]) -> Self {
        let tok_span = err.span();
        let start_tok = tok_span.start;
        let end_tok = tok_span.end;

        let char_span = if start_tok < token_spans.len() {
            let start = token_spans[start_tok].start;
            let end = if end_tok > start_tok && end_tok <= token_spans.len() {
                token_spans[end_tok - 1].end
            } else {
                start
            };
            start..end
        } else {
            let eof = token_spans.last().map_or(0, |r| r.end);
            eof..eof
        };

        Self {
            span: char_span,
            message: format_reason(err.reason()),
        }
    }

    /// Imprime no stderr um diagnóstico formatado para este erro.
    ///
    /// # Panics
    ///
    /// Causa panic se o diagnóstico não puder ser escrito no stderr.
    pub fn report(&self, file_path: &str, source: &str) {
        Report::build(ReportKind::Error, (file_path, self.span.clone()))
            .with_message(&self.message)
            .with_label(
                Label::new((file_path, self.span.clone()))
                    .with_message("aqui")
                    .with_color(Color::Red),
            )
            .finish()
            .eprint((file_path, Source::from(source)))
            .unwrap();
    }
}

fn format_pattern(pat: &RichPattern<'_, Token>) -> String {
    match pat {
        RichPattern::Token(t) => format!("'{}'", &**t),
        RichPattern::Label(l) => l.to_string(),
        RichPattern::Identifier(s) => s.clone(),
        RichPattern::Any => "qualquer token".to_string(),
        RichPattern::SomethingElse => "outro token".to_string(),
        RichPattern::EndOfInput => "fim do arquivo".to_string(),
        _ => "algo".to_string(),
    }
}

fn format_reason(reason: &RichReason<'_, Token>) -> String {
    match reason {
        RichReason::ExpectedFound { expected, found } => {
            let exp = if expected.is_empty() {
                "algo válido".to_string()
            } else {
                expected
                    .iter()
                    .map(format_pattern)
                    .collect::<Vec<_>>()
                    .join(" ou ")
            };
            found.as_ref().map_or_else(
                || format!("esperava {exp}, mas o arquivo terminou"),
                |f| format!("esperava {exp}, encontrou '{}'", &**f),
            )
        }
        RichReason::Custom(s) => s.clone(),
    }
}
