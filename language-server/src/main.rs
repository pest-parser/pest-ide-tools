use std::collections::BTreeMap;
use std::env::args;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;

use capabilities::capabilities;
use lsp::PestLanguageServerImpl;
use reqwest::ClientBuilder;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    DidChangeWatchedFilesParams, DocumentFormattingParams, InitializeParams, InitializeResult,
    InitializedParams,
};
use tower_lsp::{
    lsp_types::{
        request::{GotoDeclarationParams, GotoDeclarationResponse},
        CompletionParams, CompletionResponse, DeleteFilesParams, DidChangeTextDocumentParams,
        DidOpenTextDocumentParams, GotoDefinitionParams, GotoDefinitionResponse, Hover,
        HoverParams, Location, ReferenceParams, RenameParams, TextEdit, WorkspaceEdit,
    },
    LanguageServer,
};
use tower_lsp::{Client, LspService, Server};

mod builtins;
mod capabilities;
mod helpers;
mod lsp;

use builtins::BUILTINS;

#[derive(Debug)]
pub struct PestLanguageServer(Arc<RwLock<PestLanguageServerImpl>>);

impl PestLanguageServer {
    pub fn new(client: Client) -> Self {
        let mut cached_rule_identifiers = vec![];
        cached_rule_identifiers.extend(BUILTINS.iter().map(|s| s.to_string()));

        Self(Arc::new(RwLock::new(PestLanguageServerImpl {
            client,
            documents: BTreeMap::new(),
            cached_rule_identifiers,
        })))
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for PestLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(capabilities())
    }

    async fn initialized(&self, params: InitializedParams) {
        self.0.read().await.initialized(params).await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.0.read().await.shutdown().await
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.0.write().await.did_open(params).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.0.write().await.did_change(params).await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        self.0.write().await.did_change_watched_files(params).await;
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        self.0.write().await.did_delete_files(params).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.0.write().await.completion(params)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.0.read().await.hover(params)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        self.0.read().await.rename(params)
    }

    async fn goto_declaration(
        &self,
        params: GotoDeclarationParams,
    ) -> Result<Option<GotoDeclarationResponse>> {
        self.0.read().await.goto_declaration(params)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.0.read().await.goto_definition(params)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        self.0.read().await.references(params)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        self.0.read().await.formatting(params)
    }
}

#[tokio::main]
async fn main() {
    let args = args();
    let mut iter = args.skip(1);

    let mut check_updates = true;

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--version" => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                exit(0);
            }
            "--no-update-check" => {
                check_updates = false;
            }
            _ => eprintln!("Unexpected argument {}", arg)
        }
    }

    if check_updates {
        if let Some(new_version) = check_for_updates().await {
            println!(
                "A new version of pest_language_server is available: v{}",
                new_version
            );
        } else {
            println!("pest_language_server is up to date.");
        }
    }

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(PestLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

async fn check_for_updates() -> Option<String> {
    let client = ClientBuilder::new()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .timeout(Duration::from_secs(5))
        .build()
        .ok();

    if let Some(client) = client {
        let response = client
            .get("https://crates.io/api/v1/crates/pest_language_server")
            .send()
            .await;

        if let Ok(response) = response {
            return response
                .json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|json| {
                    let version = json["crate"]["max_version"].as_str()?;

                    if version != env!("CARGO_PKG_VERSION") {
                        Some(version.to_string())
                    } else {
                        None
                    }
                });
        }
    }

    None
}
