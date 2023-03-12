use crate::config::Config;
use log::debug;
use std::collections::HashMap;
use std::default::Default;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use sv_parser::{parse_sv_str, Define, DefineText};
use svlint::config::Config as LintConfig;
use svlint::linter::Linter;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{async_trait, Client, LanguageServer};

pub struct Backend {
    client: Client,
    root_uri: Arc<RwLock<Option<Url>>>,
    config: Arc<RwLock<Option<Config>>>,
    linter: Arc<RwLock<Option<Linter>>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Backend {
            client,
            root_uri: Default::default(),
            config: Default::default(),
            linter: Default::default(),
        }
    }

    fn lint(&self, s: &str, path: &Path) -> Vec<Diagnostic> {
        let mut ret = Vec::new();

        let root_uri = self.root_uri.read().unwrap();
        let root_uri = if let Some(ref root_uri) = *root_uri {
            if let Ok(root_uri) = root_uri.to_file_path() {
                root_uri
            } else {
                PathBuf::from("")
            }
        } else {
            PathBuf::from("")
        };

        let config = self.config.read().unwrap();
        let mut include_paths = Vec::new();
        let mut defines = HashMap::new();
        if let Some(ref config) = *config {
            for path in &config.verilog.include_paths {
                let mut p = root_uri.clone();
                p.push(PathBuf::from(path));
                include_paths.push(p);
            }
            for define in &config.verilog.defines {
                let mut define = define.splitn(2, '=');
                let ident = String::from(define.next().unwrap());
                let text = define
                    .next()
                    .and_then(|x| enquote::unescape(x, None).ok())
                    .map(|x| DefineText::new(x, None));
                let define = Define::new(ident.clone(), vec![], text);
                defines.insert(ident, Some(define));
            }
        };
        debug!("include_paths: {:?}", include_paths);
        debug!("defines: {:?}", defines);

        let src_path = if let Ok(x) = path.strip_prefix(root_uri) {
            x.to_path_buf()
        } else {
            PathBuf::from("")
        };

        let parsed = parse_sv_str(s, &src_path, &defines, &include_paths, false, false);
        match parsed {
            Ok((syntax_tree, _new_defines)) => {
                let mut linter = self.linter.write().unwrap();
                if let Some(ref mut linter) = *linter {
                    for event in syntax_tree.into_iter().event() {
                        for failed in linter.check(&syntax_tree, &event) {
                            debug!("{:?}", failed);
                            if failed.path != src_path {
                                continue;
                            }
                            let (line, col) = get_position(s, failed.beg);
                            ret.push(Diagnostic::new(
                                Range::new(
                                    Position::new(line, col),
                                    Position::new(line, col + failed.len as u32),
                                ),
                                Some(DiagnosticSeverity::WARNING),
                                Some(NumberOrString::String(failed.name)),
                                Some(String::from("svls")),
                                failed.hint,
                                None,
                                None,
                            ));
                        }
                    }
                }
            }
            Err(x) => {
                debug!("parse_error: {:?}", x);
                if let sv_parser::Error::Parse(Some((path, pos))) = x {
                    if path.as_path() == Path::new("") {
                        let (line, col) = get_position(s, pos);
                        let line_end = get_line_end(s, pos);
                        let len = line_end - pos as u32;
                        ret.push(Diagnostic::new(
                            Range::new(Position::new(line, col), Position::new(line, col + len)),
                            Some(DiagnosticSeverity::ERROR),
                            None,
                            Some(String::from("svls")),
                            String::from("parse error"),
                            None,
                            None,
                        ));
                    }
                }
            }
        }
        ret
    }
}

#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        debug!("root_uri: {:?}", params.root_uri);

        let config_svls = search_config(&PathBuf::from(".svls.toml"));
        debug!("config_svls: {:?}", config_svls);
        let config = match generate_config(config_svls) {
            Ok(x) => x,
            Err(x) => {
                self.client.show_message(MessageType::WARNING, &x).await;
                Config::default()
            }
        };

        if config.option.linter {
            let config_svlint = search_config_svlint(&PathBuf::from(".svlint.toml"));
            debug!("config_svlint: {:?}", config_svlint);

            let linter = match generate_linter(config_svlint) {
                Ok(x) => x,
                Err(x) => {
                    self.client.show_message(MessageType::WARNING, &x).await;
                    Linter::new(LintConfig::new().enable_all())
                }
            };

            let mut w = self.linter.write().unwrap();
            *w = Some(linter);
        }

        let mut w = self.root_uri.write().unwrap();
        *w = params.root_uri.clone();

        let mut w = self.config.write().unwrap();
        *w = Some(config);

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: String::from("svls"),
                version: Some(String::from(env!("CARGO_PKG_VERSION"))),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {}

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("did_open");
        let path = params.text_document.uri.to_file_path().unwrap();
        let diag = self.lint(&params.text_document.text, &path);
        self.client
            .publish_diagnostics(
                params.text_document.uri,
                diag,
                Some(params.text_document.version),
            )
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        debug!("did_change");
        let path = params.text_document.uri.to_file_path().unwrap();
        let diag = self.lint(&params.content_changes[0].text, &path);
        self.client
            .publish_diagnostics(
                params.text_document.uri,
                diag,
                Some(params.text_document.version),
            )
            .await;
    }
}

fn search_config(config: &Path) -> Option<PathBuf> {
    let curr = env::current_dir().ok()?;
    curr.ancestors().find_map(|dir| {
        let candidate = dir.join(config);
        if candidate.exists() {
            Some(candidate)
        } else {
            None
        }
    })
}

fn search_config_svlint(config: &Path) -> Option<PathBuf> {
    if let Ok(c) = env::var("SVLINT_CONFIG") {
        let candidate = Path::new(&c);
        if candidate.exists() {
            return Some(candidate.to_path_buf());
        } else {
            debug!(
                "SVLINT_CONFIG=\"{}\" does not exist. Searching hierarchically.",
                c
            );
        }
    }

    search_config(config)
}

fn generate_config(config: Option<PathBuf>) -> std::result::Result<Config, String> {
    let path = match config {
        Some(c) => c,
        _ => return Ok(Default::default()),
    };
    let text = std::fs::read_to_string(&path).map_err(|_| {
        format!(
            "Failed to read {}. Enable all lint rules.",
            path.to_string_lossy()
        )
    })?;
    toml::from_str(&text).map_err(|_| {
        format!(
            "Failed to parse {}. Enable all lint rules.",
            path.to_string_lossy()
        )
    })
}

fn generate_linter(config: Option<PathBuf>) -> std::result::Result<Linter, String> {
    let path =
        config.ok_or_else(|| String::from(".svlint.toml is not found. Enable all lint rules."))?;
    let text = std::fs::read_to_string(&path).map_err(|_| {
        format!(
            "Failed to read {}. Enable all lint rules.",
            path.to_string_lossy()
        )
    })?;
    let parsed = toml::from_str(&text).map_err(|_| {
        format!(
            "Failed to parse {}. Enable all lint rules.",
            path.to_string_lossy()
        )
    })?;
    Ok(Linter::new(parsed))
}

fn get_position(s: &str, pos: usize) -> (u32, u32) {
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

fn get_line_end(s: &str, pos: usize) -> u32 {
    let mut p = pos;
    while p < s.len() {
        if let Some(c) = s.get(p..p + 1) {
            if c == "\n" {
                break;
            }
        }
        p += 1;
    }
    p as u32
}
