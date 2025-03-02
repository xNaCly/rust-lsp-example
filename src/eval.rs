use crate::error::LspError;
use crate::lexer::TokenType;
use crate::parser::Node;
use crate::Context;

impl Context {
    fn get_var(&self, ident: &str) -> Option<&Node> {
        self.variables.get(ident)
    }

    pub fn eval(&mut self, ast: Node) -> Result<Option<Node>, LspError> {
        match ast {
            Node::Number { .. } | Node::String { .. } | Node::Null | Node::List(_) => Ok(Some(ast)),
            Node::Ident { ctx, val } => {
                let n = if let Some(node) = self.get_var(&val) {
                    node.clone()
                } else {
                    return Err(LspError::with_context(
                        ctx.into(),
                        format!("undefined identifier: {}", val),
                    ));
                };

                self.eval(n)
            }
            Node::Var { ident, value, .. } => {
                self.variables.insert(ident.to_string(), *value.clone());
                Ok(None)
            }
        }
    }

    pub fn eval_string(&mut self, ast: &Node) -> Result<Option<String>, LspError> {
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

                self.eval_string(&n)
            }
            Node::String { val, .. } => Ok(Some(format!("`{val}`"))),
            Node::List(children) => {
                let mut buf = String::new();
                buf.push('(');
                for i in 0..children.len() {
                    if let Some(eval_result) = self.eval_string(&children[i])? {
                        buf.push_str(&eval_result);
                    }
                    if i < children.len() - 1 {
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
