use tower_lsp::lsp_types::{
    CodeActionKind, CodeActionOptions, CodeActionProviderCapability, CompletionOptions,
    FileOperationFilter, FileOperationPattern, FileOperationRegistrationOptions,
    HoverProviderCapability, InitializeResult, OneOf, ServerCapabilities, ServerInfo,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    WorkDoneProgressOptions, WorkspaceFileOperationsServerCapabilities,
    WorkspaceServerCapabilities,
};

/// Returns the capabilities of the language server.
pub fn capabilities() -> InitializeResult {
    InitializeResult {
        capabilities: ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    change: Some(TextDocumentSyncKind::FULL),
                    open_close: Some(true),
                    ..Default::default()
                },
            )),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            completion_provider: Some(CompletionOptions {
                trigger_characters: Some(vec!["{".to_string(), "~".to_string(), "|".to_string()]),
                ..Default::default()
            }),
            code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
                code_action_kinds: Some(vec![
                    CodeActionKind::REFACTOR_EXTRACT,
                    CodeActionKind::REFACTOR_INLINE,
                ]),
                work_done_progress_options: WorkDoneProgressOptions::default(),
                resolve_provider: None,
                //FIXME(Jamalam): Use Default here once https://github.com/gluon-lang/lsp-types/issues/260 is resolved.
                // ..Default::default()
            })),
            definition_provider: Some(OneOf::Left(true)),
            references_provider: Some(OneOf::Left(true)),
            document_formatting_provider: Some(OneOf::Left(true)),
            rename_provider: Some(OneOf::Left(true)),
            workspace: Some(WorkspaceServerCapabilities {
                file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                    did_delete: Some(FileOperationRegistrationOptions {
                        filters: vec![FileOperationFilter {
                            pattern: FileOperationPattern {
                                glob: "**".to_string(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }],
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        },
        server_info: Some(ServerInfo {
            name: "Pest Language Server".to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }),
    }
}
