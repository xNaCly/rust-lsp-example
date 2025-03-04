use lsp_server::{Connection, ExtractError, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument},
    request::{DocumentDiagnosticRequest, HoverRequest},
    DiagnosticOptions, InitializeParams, SaveOptions, ServerCapabilities, TextDocumentSyncKind,
    TextDocumentSyncOptions,
};

use crate::{error::LspError, lexer::Lexer, parser::Node};

pub fn start() -> Result<(), String> {
    let (connection, threads) = Connection::stdio();
    let capabilities = serde_json::to_value(&ServerCapabilities {
        // hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
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

fn event_loop(connection: Connection, params: serde_json::Value) -> Result<(), String> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    let mut nodes: Vec<Node> = vec![];
    let mut errors: Vec<LspError> = vec![];

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
                                // TODO: hover here
                                continue;
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
                                continue;
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
                            let text = &(params.content_changes[0].text.clone().into_bytes());
                            let tokens = Lexer::new(&text)
                                .filter_map(|x| match x {
                                    Err(err) => {
                                        errors.push(err);
                                        None
                                    }
                                    Ok(t) => Some(t),
                                })
                                .collect::<Vec<_>>();
                            crate::parser::Parser::new(&tokens).filter_map(|x| match x {
                                Err(err) => {
                                    errors.push(err);
                                    None
                                }
                                Ok(t) => Some(t),
                            });
                        }
                        Err(err) => panic!("failed to cast notification: {err:?}"),
                    };
                }
                "textDocument/didOpen" => {
                    match cast_noti::<DidOpenTextDocument>(not) {
                        Ok(params) => {
                            let text = &(params.text_document.text.into_bytes());
                            let tokens = Lexer::new(&text)
                                .filter_map(|x| match x {
                                    Err(err) => {
                                        errors.push(err);
                                        None
                                    }
                                    Ok(t) => Some(t),
                                })
                                .collect::<Vec<_>>();
                            crate::parser::Parser::new(&tokens).filter_map(|x| match x {
                                Err(err) => {
                                    errors.push(err);
                                    None
                                }
                                Ok(t) => Some(t),
                            });
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
