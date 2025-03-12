use crate::error::LspError;
use crate::lexer::TokenType;
use crate::parser::Node;
use crate::Context;

impl Context {
    pub fn clear(&mut self) {
        self.variables.clear();
        self.types_on_line.clear();
    }

    fn get_var(&self, ident: &str) -> Option<&Node> {
        self.variables.get(ident)
    }

    pub fn eval(&mut self, ast: Node) -> Result<Option<Node>, LspError> {
        match ast {
            Node::Number { .. } | Node::String { .. } | Node::Null => Ok(Some(ast)),
            Node::List { val, ctx } => {
                let v = Vec::with_capacity(val.len());
                for node in val {
                    self.eval(node)?;
                }
                Ok(Some(Node::List { ctx, val: v }))
            }
            Node::Ident { ctx, val } => {
                let n = if let Some(node) = self.get_var(&val) {
                    node
                } else {
                    return Err(LspError::with_context(
                        ctx.into(),
                        format!("undefined identifier: `{val}`"),
                    ));
                };

                self.eval(n.clone())
            }
            Node::Var { ident, val, .. } => {
                self.variables.insert(ident.to_string(), *val);
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
                        format!("undefined identifier: `{val}`"),
                    ));
                };

                self.eval_string(&n)
            }
            Node::String { val, .. } => Ok(Some(format!("`{val}`"))),
            Node::List { val, .. } => {
                let mut buf = String::new();
                buf.push('(');
                for i in 0..val.len() {
                    if let Some(eval_result) = self.eval_string(&val[i])? {
                        buf.push_str(&eval_result);
                    }
                    if i < val.len() - 1 {
                        buf.push_str(", ");
                    }
                }
                buf.push(')');
                Ok(Some(buf))
            }
            Node::Var {
                ident, val: value, ..
            } => {
                self.variables.insert(ident.to_string(), *value.clone());
                Ok(None)
            }
            Node::Null => Ok(Some("Null".into())),
        }
    }
}
