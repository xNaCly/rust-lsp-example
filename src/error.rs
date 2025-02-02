use crate::parser::TokenContext;

#[derive(Debug)]
pub struct LspError {
    ctx: TokenContext,
    message: String,
}

impl LspError {
    pub fn with_context(ctx: TokenContext, message: String) -> Self {
        Self { ctx, message }
    }
}
