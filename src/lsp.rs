use std::{collections::HashMap, str::FromStr};

use lsp_server::{Connection, ExtractError, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument},
    request::{DocumentDiagnosticRequest, GotoDefinition, HoverRequest},
    DiagnosticOptions, InitializeParams, Location, Position, Range, SaveOptions,
    ServerCapabilities, TextDocumentSyncKind, TextDocumentSyncOptions, Uri,
    WorkDoneProgressOptions,
};

use crate::{
    error::LspError,
    eval::Context,
    lexer::Lexer,
    parser::{Node, TokenContext},
};

#[derive(Debug)]
struct LspContext {
    pub uri: Uri,
    pub var_to_def: HashMap<String, Node>,
    pub nodes_by_line: HashMap<usize, Vec<Node>>,
}

impl Default for LspContext {
    fn default() -> Self {
        Self {
            uri: Uri::from_str("file://.").unwrap(),
            var_to_def: HashMap::new(),
            nodes_by_line: HashMap::new(),
        }
    }
}

impl LspContext {
    pub fn clear(&mut self) {
        self.nodes_by_line.clear()
    }

    pub fn query_nodes_by_pos(&self, pos: &Position) -> Option<&Node> {
        if let Some(by_line) = self.nodes_by_line.get(&(pos.line as usize)) {
            let char = (pos.character as usize);
            by_line
                .iter()
                .filter(|n| {
                    n.ctx()
                        .is_some_and(|ctx| ctx.start <= char && ctx.end >= char)
                })
                .last()
        } else {
            None
        }
    }

    pub fn from_ast(&mut self, node: Node) {
        if let Some(&TokenContext { line, .. }) = node.ctx() {
            let nodes = match self.nodes_by_line.get(&line).cloned() {
                Some(nodes) => {
                    let mut t = nodes;
                    t.push(node.clone());
                    t
                }
                None => vec![node.clone()],
            };
            self.nodes_by_line.insert(line, nodes);
        }

        match node.clone() {
            // noop, we already handle these above
            Node::Null | Node::Number { .. } | Node::String { .. } | Node::Ident { .. } => (),
            Node::Lambda { params, body, .. } => params
                .into_iter()
                .chain(body)
                .for_each(|n| self.from_ast(n)),
            Node::List { val, .. } => {
                for node in val {
                    self.from_ast(node);
                }
            }
            Node::Var { ident, val, .. } => {
                self.from_ast(*val);
                self.var_to_def.insert(ident.clone(), node);
            }
        }
    }
}

pub fn start() -> Result<(), String> {
    eprintln!("Hello from the rust-example-lsp starting point, if you can read this, the language server is attached");
    let (connection, threads) = Connection::stdio();
    let capabilities = serde_json::to_value(&ServerCapabilities {
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        definition_provider: Some(lsp_types::OneOf::Left(true)),
        diagnostic_provider: Some(lsp_types::DiagnosticServerCapabilities::Options(
            DiagnosticOptions {
                inter_file_dependencies: false,
                workspace_diagnostics: false,
                ..Default::default()
            },
        )),
        text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::FULL),
                save: Some(lsp_types::TextDocumentSyncSaveOptions::SaveOptions(
                    SaveOptions {
                        include_text: Some(true),
                    },
                )),
                ..Default::default()
            },
        )),
        ..Default::default()
    })
    .map_err(|_| "failed to serialize lsp_types::ServerCapabilities")?;

    let init_params = match connection.initialize(capabilities) {
        Ok(params) => params,
        Err(e) => {
            if e.channel_is_disconnected() {
                threads
                    .join()
                    .map_err(|_| "failed to wait on thread joining")?;
            }
            return Err(e.to_string());
        }
    };

    event_loop(connection, init_params)?;

    threads
        .join()
        .map_err(|_| "failed to wait on thread joining")?;

    Ok(())
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

fn cast_noti<N>(not: Notification) -> Result<N::Params, ExtractError<Notification>>
where
    N: lsp_types::notification::Notification,
    N::Params: serde::de::DeserializeOwned,
{
    not.extract(N::METHOD)
}

fn update_state(
    nodes: &mut Vec<Node>,
    errors: &mut Vec<LspError>,
    lsp_ctx: &mut LspContext,
    ctx: &mut Context,
    input: &[u8],
) -> Result<(), String> {
    nodes.clear();
    errors.clear();
    ctx.clear();
    let tokens = Lexer::new(input)
        .filter_map(|x| match x {
            Err(err) => {
                errors.push(err);
                None
            }
            Ok(t) => Some(t),
        })
        .collect::<Vec<_>>();
    let mut ast = crate::parser::Parser::new(&tokens)
        .filter_map(|x| match x {
            Err(err) => {
                errors.push(err);
                None
            }
            Ok(t) => Some(t),
        })
        .collect::<Vec<_>>();

    nodes.append(&mut ast.clone());
    ast.into_iter().for_each(|n| {
        lsp_ctx.from_ast(n.clone());
        ctx.eval(n);
    });
    errors.append(&mut ctx.errors);
    Ok(())
}

fn event_loop(connection: Connection, params: serde_json::Value) -> Result<(), String> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    let mut nodes: Vec<Node> = vec![];
    let mut errors: Vec<LspError> = vec![];
    let mut ctx = Context::default();
    let mut lsp_ctx = LspContext::default();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection
                    .handle_shutdown(&req)
                    .map_err(|e| e.to_string())?
                {
                    return Ok(());
                }
                match req.method.as_str() {
                    "textDocument/definition" => match cast::<GotoDefinition>(req) {
                        Ok((id, params)) => {
                            let node_at_location = lsp_ctx
                                .query_nodes_by_pos(&params.text_document_position_params.position);
                            if let Some(Node::Ident { val, .. }) = node_at_location {
                                if let Some(source) = lsp_ctx.var_to_def.get(val) {
                                    let (start, end) = source
                                        .ctx()
                                        .map(|ctx| {
                                            (
                                                Position {
                                                    line: ctx.line as u32,
                                                    character: ctx.start as u32,
                                                },
                                                Position {
                                                    line: ctx.line as u32,
                                                    character: ctx.end as u32,
                                                },
                                            )
                                        })
                                        .unwrap_or_else(|| {
                                            (Position::default(), Position::default())
                                        });
                                    let location = Location {
                                        range: Range { start, end },
                                        uri: lsp_ctx.uri.clone(),
                                    };
                                    let result = serde_json::to_value(&location).unwrap();
                                    let resp = Response {
                                        id,
                                        result: Some(result),
                                        error: None,
                                    };
                                    connection
                                        .sender
                                        .send(Message::Response(resp))
                                        .map_err(|_| "failed to send hover response")?;
                                }
                            };
                        }
                        Err(err) => panic!("{err:?}"),
                    },
                    "textDocument/hover" => {
                        match cast::<HoverRequest>(req) {
                            Ok((id, params)) => {
                                let node = lsp_ctx.query_nodes_by_pos(
                                    &params.text_document_position_params.position,
                                );
                                let text: String = match node {
                                    Some(node) => node
                                        .node_type(&mut ctx)
                                        .map_err(|e| errors.push(e))
                                        .unwrap_or_else(|_| "Unknown".into()),
                                    _ => "Unknown".into(),
                                };
                                let hover_result = lsp_types::Hover {
                                    contents: lsp_types::HoverContents::Scalar(
                                        lsp_types::MarkedString::String(text),
                                    ),
                                    range: None,
                                };
                                let result = serde_json::to_value(&hover_result).unwrap();
                                let resp = Response {
                                    id,
                                    result: Some(result),
                                    error: None,
                                };
                                connection
                                    .sender
                                    .send(Message::Response(resp))
                                    .map_err(|_| "failed to send hover response")?;
                            }
                            Err(err) => panic!("{err:?}"),
                        };
                    }
                    "textDocument/diagnostic" => {
                        match cast::<DocumentDiagnosticRequest>(req) {
                            Ok((id, params)) => {
                                let diagnostics = lsp_types::FullDocumentDiagnosticReport {
                                    result_id: None,
                                    items: errors.iter().map(|e| e.into()).collect(),
                                };
                                let result = serde_json::to_value(&diagnostics).unwrap();
                                let resp = Response {
                                    id,
                                    result: Some(result),
                                    error: None,
                                };
                                connection
                                    .sender
                                    .send(Message::Response(resp))
                                    .map_err(|_| "failed to send diagnostics")?;
                            }
                            Err(err) => panic!("{err:?}"),
                        };
                    }
                    _ => (),
                }
                // ...
            }
            Message::Response(resp) => {
                eprintln!("got response: {resp:?}");
            }
            Message::Notification(not) => match not.method.as_str() {
                "textDocument/didChange" => {
                    match cast_noti::<DidChangeTextDocument>(not) {
                        Ok(params) => {
                            lsp_ctx.uri = params.text_document.uri;
                            update_state(
                                &mut nodes,
                                &mut errors,
                                &mut lsp_ctx,
                                &mut ctx,
                                &(params.content_changes[0].text.as_bytes()),
                            );
                        }
                        Err(err) => panic!("failed to cast notification: {err:?}"),
                    };
                }
                "textDocument/didOpen" => {
                    match cast_noti::<DidOpenTextDocument>(not) {
                        Ok(params) => {
                            lsp_ctx.uri = params.text_document.uri;
                            update_state(
                                &mut nodes,
                                &mut errors,
                                &mut lsp_ctx,
                                &mut ctx,
                                &(params.text_document.text.into_bytes()),
                            )
                        }
                        Err(err) => panic!("failed to cast notification: {err:?}"),
                    };
                }
                _ => (),
            },
        }
    }
    Ok(())
}
