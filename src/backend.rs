use crate::config::Config;
use futures::future;
use jsonrpc_core::{BoxFuture, Result};
use log::debug;
use serde_json::Value;
use std::collections::HashMap;
use std::default::Default;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use sv_parser::parse_sv_str;
use svlint::config::Config as LintConfig;
use svlint::linter::Linter;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, Printer};

#[derive(Default)]
pub struct Backend {
    root_path: Arc<RwLock<Option<String>>>,
    root_uri: Arc<RwLock<Option<Url>>>,
    config: Arc<RwLock<Option<Config>>>,
    linter: Arc<RwLock<Option<Linter>>>,
}

impl Backend {
    fn lint(&self, s: &str) -> Vec<Diagnostic> {
        let mut ret = Vec::new();

        let root_path = self.root_path.read().unwrap();
        let root_path = if let Some(ref root_path) = *root_path {
            PathBuf::from(root_path)
        } else {
            PathBuf::from("")
        };

        let config = self.config.read().unwrap();
        let mut include_paths = Vec::new();
        if let Some(ref config) = *config {
            for path in &config.verilog.include_paths {
                let mut p = root_path.clone();
                p.push(PathBuf::from(path));
                include_paths.push(p);
            }
        };
        debug!("include_paths: {:?}", include_paths);

        let parsed = parse_sv_str(s, &PathBuf::from(""), &HashMap::new(), &include_paths);
        match parsed {
            Ok((syntax_tree, _new_defines)) => {
                let linter = self.linter.read().unwrap();
                if let Some(ref linter) = *linter {
                    for node in &syntax_tree {
                        for failed in linter.check(&syntax_tree, &node) {
                            debug!("{:?}", failed);
                            if failed.path != PathBuf::from("") {
                                continue;
                            }
                            let (line, col) = get_position(s, failed.beg);
                            ret.push(Diagnostic::new(
                                Range::new(
                                    Position::new(line, col),
                                    Position::new(line, col + failed.len as u64),
                                ),
                                Some(DiagnosticSeverity::Warning),
                                Some(NumberOrString::String(String::from(failed.name))),
                                Some(String::from("svls")),
                                String::from(failed.hint),
                                None,
                            ));
                        }
                    }
                }
            }
            Err(x) => {
                debug!("parse_error: {:?}", x);
                match x.kind() {
                    sv_parser::ErrorKind::Parse(Some((path, pos))) => {
                        if path == &PathBuf::from("") {
                            let pos = *pos;
                            let (line, col) = get_position(s, pos);
                            let line_end = get_line_end(s, pos);
                            let len = line_end - pos as u64;
                            ret.push(Diagnostic::new(
                                Range::new(
                                    Position::new(line, col),
                                    Position::new(line, col + len),
                                ),
                                Some(DiagnosticSeverity::Error),
                                None,
                                Some(String::from("svls")),
                                String::from("parse error"),
                                None,
                            ));
                        }
                    }
                    _ => (),
                }
            }
        }
        ret
    }
}

impl LanguageServer for Backend {
    type ShutdownFuture = BoxFuture<()>;
    type SymbolFuture = BoxFuture<Option<Vec<SymbolInformation>>>;
    type ExecuteFuture = BoxFuture<Option<Value>>;
    type CompletionFuture = BoxFuture<Option<CompletionResponse>>;
    type HoverFuture = BoxFuture<Option<Hover>>;
    type HighlightFuture = BoxFuture<Option<Vec<DocumentHighlight>>>;

    fn initialize(&self, printer: &Printer, params: InitializeParams) -> Result<InitializeResult> {
        debug!("root_path: {:?}", params.root_path);
        debug!("root_uri: {:?}", params.root_uri);

        let config_svls = search_config(&PathBuf::from(".svls.toml"));
        debug!("config_svls: {:?}", config_svls);
        let config = match generate_config(config_svls) {
            Ok(x) => x,
            Err(x) => {
                printer.show_message(MessageType::Warning, &x);
                Config::default()
            }
        };

        if config.option.linter {
            let config_svlint = search_config(&PathBuf::from(".svlint.toml"));
            debug!("config_svlint: {:?}", config_svlint);

            let linter = match generate_linter(config_svlint) {
                Ok(x) => x,
                Err(x) => {
                    printer.show_message(MessageType::Warning, &x);
                    Linter::new(LintConfig::new().enable_all())
                }
            };

            let mut w = self.linter.write().unwrap();
            *w = Some(linter);
        }

        let mut w = self.root_path.write().unwrap();
        *w = params.root_path.clone();

        let mut w = self.root_uri.write().unwrap();
        *w = params.root_uri.clone();

        let mut w = self.config.write().unwrap();
        *w = Some(config);

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::Full,
                )),
                hover_provider: Some(true),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                }),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: None,
                }),
                document_highlight_provider: Some(false),
                workspace_symbol_provider: Some(true),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["dummy.do_something".to_string()],
                }),
                workspace: Some(WorkspaceCapability {
                    workspace_folders: Some(WorkspaceFolderCapability {
                        supported: Some(true),
                        change_notifications: Some(
                            WorkspaceFolderCapabilityChangeNotifications::Bool(true),
                        ),
                    }),
                }),
                ..ServerCapabilities::default()
            },
        })
    }

    fn initialized(&self, printer: &Printer, _: InitializedParams) {
        printer.log_message(MessageType::Info, &format!("server initialized"));
    }

    fn shutdown(&self) -> Self::ShutdownFuture {
        Box::new(future::ok(()))
    }

    fn symbol(&self, _: WorkspaceSymbolParams) -> Self::SymbolFuture {
        Box::new(future::ok(None))
    }

    fn did_change_workspace_folders(&self, _: &Printer, _: DidChangeWorkspaceFoldersParams) {}

    fn did_change_configuration(&self, _: &Printer, _: DidChangeConfigurationParams) {}

    fn did_change_watched_files(&self, _: &Printer, _: DidChangeWatchedFilesParams) {}

    fn execute_command(&self, _: &Printer, _: ExecuteCommandParams) -> Self::ExecuteFuture {
        Box::new(future::ok(None))
    }

    fn did_open(&self, printer: &Printer, params: DidOpenTextDocumentParams) {
        let diag = self.lint(&params.text_document.text);
        printer.publish_diagnostics(params.text_document.uri, diag);
    }

    fn did_change(&self, printer: &Printer, params: DidChangeTextDocumentParams) {
        let diag = self.lint(&params.content_changes[0].text);
        printer.publish_diagnostics(params.text_document.uri, diag);
    }

    fn did_save(&self, _: &Printer, _: DidSaveTextDocumentParams) {}

    fn did_close(&self, _: &Printer, _: DidCloseTextDocumentParams) {}

    fn completion(&self, _: CompletionParams) -> Self::CompletionFuture {
        Box::new(future::ok(None))
    }

    fn hover(&self, _: TextDocumentPositionParams) -> Self::HoverFuture {
        Box::new(future::ok(None))
    }

    fn document_highlight(&self, _: TextDocumentPositionParams) -> Self::HighlightFuture {
        Box::new(future::ok(None))
    }
}

fn search_config(config: &Path) -> Option<PathBuf> {
    if let Ok(current) = env::current_dir() {
        for dir in current.ancestors() {
            let candidate = dir.join(config);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        None
    } else {
        None
    }
}

fn generate_config(config: Option<PathBuf>) -> std::result::Result<Config, String> {
    if let Some(config) = config {
        if let Ok(s) = std::fs::read_to_string(&config) {
            if let Ok(config) = toml::from_str(&s) {
                Ok(config)
            } else {
                Err(format!(
                    "Failed to parse {}. Enable all lint rules.",
                    config.to_string_lossy()
                ))
            }
        } else {
            Err(format!(
                "Failed to read {}. Enable all lint rules.",
                config.to_string_lossy()
            ))
        }
    } else {
        Ok(Config::default())
    }
}

fn generate_linter(config: Option<PathBuf>) -> std::result::Result<Linter, String> {
    if let Some(config) = config {
        if let Ok(s) = std::fs::read_to_string(&config) {
            if let Ok(config) = toml::from_str(&s) {
                Ok(Linter::new(config))
            } else {
                Err(format!(
                    "Failed to parse {}. Enable all lint rules.",
                    config.to_string_lossy()
                ))
            }
        } else {
            Err(format!(
                "Failed to read {}. Enable all lint rules.",
                config.to_string_lossy()
            ))
        }
    } else {
        Err(format!(".svlint.toml is not found. Enable all lint rules."))
    }
}

fn get_position(s: &str, pos: usize) -> (u64, u64) {
    let mut line = 0;
    let mut col = 0;
    let mut p = 0;
    while p < pos {
        if let Some(c) = s.get(p..p + 1) {
            if c == "\n" {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        } else {
            col += 1;
        }
        p += 1;
    }
    (line, col)
}

fn get_line_end(s: &str, pos: usize) -> u64 {
    let mut p = pos;
    while p < s.len() {
        if let Some(c) = s.get(p..p + 1) {
            if c == "\n" {
                break;
            }
        }
        p += 1;
    }
    p as u64
}
