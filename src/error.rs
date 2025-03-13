use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::parser::TokenContext;

#[derive(Debug, Clone)]
pub struct LspError {
    pub ctx: TokenContext,
    pub message: String,
}

impl LspError {
    pub fn with_context(ctx: TokenContext, message: impl Into<String>) -> Self {
        Self {
            ctx,
            message: message.into(),
        }
    }
}

impl From<&LspError> for Diagnostic {
    fn from(value: &LspError) -> Self {
        Self {
            range: Range::new(
                Position {
                    line: value.ctx.line as u32,
                    character: value.ctx.start as u32,
                },
                Position {
                    line: value.ctx.line as u32,
                    character: value.ctx.end as u32,
                },
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(lsp_types::NumberOrString::String(String::from(
                "rust-lsp-example",
            ))),
            code_description: None,
            source: Some("rust-lsp-example".into()),
            message: value.message.clone(),
            related_information: None,
            tags: None,
            data: None,
        }
    }
}
