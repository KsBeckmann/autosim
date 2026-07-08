use ariadne::{Color, Label, Report, ReportKind, Source};
use std::{error, fmt, ops};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LexError {
    pub span: ops::Range<usize>,
    pub text: String,
}

impl LexError {
    /// Imprime no stderr um diagnóstico formatado para este erro.
    ///
    /// # Panics
    ///
    /// Causa panic se o diagnóstico não puder ser escrito no stderr.
    pub fn report(&self, file_path: &str, source: &str) {
        let span = self.span.clone();
        Report::build(ReportKind::Error, (file_path, span.clone()))
            .with_message(format!("token inválido `{}`", self.text))
            .with_label(
                Label::new((file_path, span))
                    .with_message("token desconhecido")
                    .with_color(Color::Red),
            )
            .finish()
            .eprint((file_path, Source::from(source)))
            .unwrap();
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "token inválido `{}` em {}..{}",
            self.text, self.span.start, self.span.end
        )
    }
}

impl error::Error for LexError {}
