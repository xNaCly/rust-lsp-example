use std::char;

use lsp_server::{Connection, ExtractError, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument},
    request::{DocumentDiagnosticRequest, HoverRequest},
    DiagnosticOptions, InitializeParams, Position, SaveOptions, ServerCapabilities,
    TextDocumentSyncKind, TextDocumentSyncOptions,
};

use crate::{
    error::LspError,
    lexer::Lexer,
    parser::{Context, Node, TokenContext},
};

pub fn start() -> Result<(), String> {
    let (connection, threads) = Connection::stdio();
    let capabilities = serde_json::to_value(&ServerCapabilities {
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
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

    let mut ctx = Context::default();
    nodes.append(&mut ast.clone());
    ast.into_iter().for_each(|node| {
        if let Err(e) = ctx.eval(node) {
            errors.push(e);
        }
    });
    Ok(())
}

fn event_loop(connection: Connection, params: serde_json::Value) -> Result<(), String> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    let mut nodes: Vec<Node> = vec![];
    let mut errors: Vec<LspError> = vec![];
    let mut ctx = Context::default();

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
                    "textDocument/hover" => {
                        match cast::<HoverRequest>(req) {
                            Ok((id, params)) => {
                                let Position { line, character } =
                                    params.text_document_position_params.position;
                                let (character, line) = (character as usize, line as usize);
                                // TODO: make this work for non top level nodes
                                let cur = nodes
                                    .iter()
                                    .filter(|n| {
                                        n.ctx().is_some_and(|ctx| {
                                            ctx.line == line
                                                && ctx.start <= character
                                                && ctx.end >= character
                                        })
                                    })
                                    .last()
                                    .map(|n| n.node_type(&mut ctx).unwrap_or_default())
                                    .unwrap_or_default();
                                let hover_result = lsp_types::Hover {
                                    contents: lsp_types::HoverContents::Scalar(
                                        lsp_types::MarkedString::String(cur),
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
                                    .map_err(|_| "failed to send definition")?;
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
                        Ok(params) => update_state(
                            &mut nodes,
                            &mut errors,
                            &mut ctx,
                            &(params.content_changes[0].text.as_bytes()),
                        ),
                        Err(err) => panic!("failed to cast notification: {err:?}"),
                    };
                }
                "textDocument/didOpen" => {
                    match cast_noti::<DidOpenTextDocument>(not) {
                        Ok(params) => update_state(
                            &mut nodes,
                            &mut errors,
                            &mut ctx,
                            &(params.text_document.text.into_bytes()),
                        ),
                        Err(err) => panic!("failed to cast notification: {err:?}"),
                    };
                }
                _ => (),
            },
        }
    }
    Ok(())
}
