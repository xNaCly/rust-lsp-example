use std::collections::HashMap;

use crate::error::LspError;
use crate::lexer::TokenType;
use crate::parser::Node;

#[derive(Default, Debug)]
pub struct Context {
    pub variables: HashMap<String, Node>,
    pub types_on_line: HashMap<usize, Vec<Node>>,
    pub errors: Vec<LspError>,
}

impl Context {
    pub fn clear(&mut self) {
        self.variables.clear();
        self.types_on_line.clear();
    }

    fn get_var(&self, ident: &str) -> Option<&Node> {
        self.variables.get(ident)
    }

    fn put_var(&mut self, ident: String, val: Node) {
        self.variables.insert(ident, val);
    }

    pub fn eval(&mut self, ast: Node) -> Option<Node> {
        match ast {
            Node::Number { .. } | Node::String { .. } | Node::Null => Some(ast),
            Node::List { val, ctx } => {
                if val.len() == 0 {
                    return Some(Node::List { ctx, val: vec![] });
                }

                let first = self.eval(val[0].clone())?;

                // edge case of calling a lambda
                if let Node::Lambda { params, body, ctx } = first {
                    let mut private_scope = Context::default();
                    let args = &val[1..val.len()];
                    if args.len() != params.len() {
                        self.errors.push(LspError::with_context(
                            ctx.clone(),
                            format!(
                                "Incompatible amount of parameters ({}) and arguments ({})",
                                params.len(),
                                args.len()
                            ),
                        ));
                        return None;
                    }
                    for (i, n) in params.into_iter().enumerate() {
                        if let Node::Ident { val, .. } = n {
                            private_scope.put_var(val.to_string(), args[i].clone());
                        }
                    }

                    if body.is_empty() {
                        return Some(Node::Null);
                    }
                    for i in 0..body.len() {
                        let r = private_scope.eval(body[i].clone());
                        // last element in body has to be returned
                        if i == body.len() - 1 {
                            return r;
                        }
                    }

                    return Some(Node::Null);
                }

                let mut v = Vec::with_capacity(val.len());
                for i in 0..val.len() {
                    if let Some(node) = self.eval(val[i].clone()) {
                        v.push(node);
                    };
                }
                Some(Node::List { ctx, val: v })
            }
            Node::Lambda { .. } => Some(ast),
            Node::Ident { ctx, val } => {
                let n = if let Some(node) = self.get_var(&val) {
                    node
                } else {
                    self.errors.push(LspError::with_context(
                        ctx,
                        format!("Undefined identifier: `{val}`"),
                    ));
                    return None;
                };

                self.eval(n.clone())
            }
            Node::Var { ident, val, .. } => {
                let evaled = self.eval(*val)?;
                self.variables.insert(ident.to_string(), evaled.clone());
                Some(evaled)
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
                let mut buf = format!("{}#", val.len());
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
            Node::Lambda { params, .. } => {
                let mut s = String::from("λ(");
                for i in 0..params.len() {
                    s.push('α');
                    if i < params.len() - 1 {
                        s.push(',');
                    }
                }
                s.push(')');
                return Ok(Some(s));
            }
            Node::Null => Ok(Some("Null".into())),
        }
    }
}
