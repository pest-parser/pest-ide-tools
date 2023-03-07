use std::collections::BTreeMap;
use std::sync::Arc;

use capabilities::capabilities;
use lsp::PestLanguageServerImpl;
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
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(PestLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
