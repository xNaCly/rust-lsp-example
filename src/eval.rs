use crate::error::LspError;
use crate::lexer::TokenType;
use crate::parser::Node;
use crate::Context;

// TODO: errors require token information on ast
pub fn eval(ast: &Node, ctx: &mut Context) -> Result<String, LspError> {
    match ast {
        Node::Number(num) => Ok(num.to_string()),
        Node::Ident(ident) => {
            let node = ctx.variables.get(ident).ok_or_else(|| {
                LspError::new(0, 0, 0, format!("undefined identifier: {}", ident))
            })?;
            eval(ast, ctx)
        }
        Node::String(string) => Ok(string.to_string()),
        Node::List(children) => todo!("{:?}", children),
        Node::Null => Ok("".into()),
    }
}
