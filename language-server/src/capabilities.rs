use tower_lsp::lsp_types::{
    CodeActionKind, CodeActionOptions, CodeActionProviderCapability, CompletionOptions,
    FileOperationFilter, FileOperationPattern, FileOperationRegistrationOptions,
    HoverProviderCapability, InitializeResult, OneOf, ServerCapabilities, ServerInfo,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    WorkspaceFileOperationsServerCapabilities, WorkspaceServerCapabilities
};

/// Returns the capabilities of the language server.
pub fn capabilities() -> InitializeResult {
    let text_document_sync = Some(TextDocumentSyncCapability::Options(
        TextDocumentSyncOptions {
            change: Some(TextDocumentSyncKind::FULL),
            open_close: Some(true),
            ..Default::default()
        }
    ));

    let trigger_characters = Some(vec![
        "{".to_string(),
        "~".to_string(),
        "|".to_string(),
        "(".to_string(),
    ]);

    let completion_provider = Some(CompletionOptions {
        trigger_characters,
        ..Default::default()
    });

    let code_action_kinds = Some(vec![
        CodeActionKind::REFACTOR_EXTRACT,
        CodeActionKind::REFACTOR_INLINE,
    ]);

    let code_action_provider = Some(CodeActionProviderCapability::Options(CodeActionOptions {
        code_action_kinds,
        ..Default::default()
    }));

    let filters = vec![FileOperationFilter {
        pattern: FileOperationPattern {
            glob: "** /*.pest".to_string(),
            ..Default::default()
        },
        ..Default::default()
    }];

    let operation_options = Some(FileOperationRegistrationOptions { filters });
    let file_operations = Some(WorkspaceFileOperationsServerCapabilities {
        did_delete: operation_options.clone(),
        did_create: operation_options.clone(),
        did_rename: operation_options.clone(),
        ..Default::default()
    });

    let workspace = Some(WorkspaceServerCapabilities {
        file_operations,
        ..Default::default()
    });

    let server_info = Some(ServerInfo {
        name: "Pest Language Server".to_string(),
        version: Some(env!("CARGO_PKG_VERSION").to_string())
    });

    let capabilities = ServerCapabilities {
        text_document_sync,
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider,
        code_action_provider,
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        document_formatting_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Left(true)),
        workspace,
        ..Default::default()
    };

    InitializeResult {
        capabilities,
        server_info
    }
}
