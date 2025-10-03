use std::io::{stdin, stdout};

use capabilities::capabilities;
use clap::command;
use lsp::PestLanguageServerImpl;
use smol::{Unblock, lock::RwLock};
use tower_lsp::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::Result,
    lsp_types::{
        CodeActionParams, CodeActionResponse, CompletionParams, CompletionResponse,
        DeleteFilesParams, DidChangeConfigurationParams, DidChangeTextDocumentParams,
        DidOpenTextDocumentParams, DocumentFormattingParams, DocumentSymbolParams,
        DocumentSymbolResponse, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams,
        InitializeParams, InitializeResult, InitializedParams, Location, ReferenceParams,
        RenameParams, TextEdit, WorkspaceEdit,
        request::{GotoDeclarationParams, GotoDeclarationResponse},
    },
};

mod analysis;
mod builtins;
mod capabilities;
mod helpers;
mod lsp;

#[derive(Debug)]
/// The async-ready language server. You probably want [PestLanguageServerImpl] instead.
pub struct PestLanguageServer(RwLock<PestLanguageServerImpl>);

impl PestLanguageServer {
    pub fn new(client: Client) -> Self {
        Self(RwLock::new(PestLanguageServerImpl::new(client)))
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for PestLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(capabilities())
    }

    async fn initialized(&self, params: InitializedParams) {
        self.0.write().await.initialized(params).await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.0.read().await.shutdown().await
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        self.0.write().await.did_change_configuration(params).await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.0.write().await.did_open(params).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.0.write().await.did_change(params).await;
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        self.0.write().await.did_delete_files(params).await;
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        Ok(Some(self.0.read().await.code_action(params).await))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(self.0.read().await.completion(params))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        Ok(self.0.read().await.hover(params))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        Ok(Some(self.0.read().await.rename(params)))
    }

    async fn goto_declaration(
        &self,
        params: GotoDeclarationParams,
    ) -> Result<Option<GotoDeclarationResponse>> {
        let declaration = self
            .0
            .read()
            .await
            .goto_definition(params.text_document_position_params)
            .map(GotoDeclarationResponse::Scalar);
        Ok(declaration)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let definition = self
            .0
            .read()
            .await
            .goto_definition(params.text_document_position_params)
            .map(GotoDefinitionResponse::Scalar);
        Ok(definition)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        Ok(self.0.read().await.references(params))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        Ok(self.0.read().await.formatting(params))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        Ok(self.0.read().await.document_symbol(params))
    }
}

fn main() {
    let _args = command!().get_matches();

    let stdin = Unblock::new(stdin());
    let stdout = Unblock::new(stdout());

    let (service, socket) = LspService::new(PestLanguageServer::new);
    smol::block_on(Server::new(stdin, stdout, socket).serve(service));
}
