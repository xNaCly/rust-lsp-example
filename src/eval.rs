use crate::error::LspError;
use crate::lexer::TokenType;
use crate::parser::Node;
use crate::Context;

impl Context {
    fn get_var(&self, ident: &str) -> Option<&Node> {
        self.variables.get(ident)
    }

    pub fn eval(&mut self, ast: &Node) -> Result<Option<String>, LspError> {
        match ast {
            Node::Number { val, .. } => Ok(Some(val.to_string())),
            Node::Ident { ctx, val } => {
                let n = if let Some(node) = self.get_var(val) {
                    node.clone()
                } else {
                    return Err(LspError::with_context(
                        ctx.into(),
                        format!("undefined identifier: {}", val),
                    ));
                };

                self.eval(&n)
            }
            Node::String { val, .. } => Ok(Some(val.to_string())),
            Node::List(children) => {
                let mut buf = String::new();
                buf.push('(');
                for i in 0..children.len() {
                    if let Some(eval_result) = self.eval(&children[i])? {
                        buf.push_str(&eval_result);
                    }
                    if i < children.len() {
                        buf.push_str(", ");
                    }
                }
                buf.push(')');
                Ok(Some(buf))
            }
            Node::Var { ident, value, .. } => {
                self.variables.insert(ident.to_string(), *value.clone());
                Ok(None)
            }
            Node::Null => Ok(Some("<nil>".into())),
        }
    }
}
