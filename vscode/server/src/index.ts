/* eslint-disable @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unused-vars, node/no-unpublished-import */

import {
	createConnection,
	ProposedFeatures,
	PublishDiagnosticsParams,
	TextDocumentSyncKind,
} from "vscode-languageserver/node";
import { PestLanguageServer } from "../../build/pest_language_server";

function log(message: string) {
	console.log(`[Pest LS (TS)] ${message}`);
}

// Create LSP connection
const connection = createConnection(ProposedFeatures.all);

const diagnosticsCallback = (params: PublishDiagnosticsParams) =>
	connection.sendDiagnostics(params);

let ls = new PestLanguageServer(diagnosticsCallback);

connection.onNotification((...args) => {
	ls.onFileNotification(...args);
});

connection.onRenameRequest((params, _a, _b, _c) => {
	return ls.onRename(params);
});

connection.onHover(params => {
	return ls.onHover(params);
});

connection.onDeclaration(params => {
	return ls.onGotoDeclaration(params);
});

connection.onDefinition(params => {
	return ls.onGotoDefinition(params);
});

connection.onReferences(params => {
	return ls.onFindReferences(params);
});

connection.onCompletion(params => {
	return ls.onCompletion(params);
});

connection.onDocumentFormatting(params => {
	return ls.onDocumentFormatting(params);
});

connection.onExecuteCommand(params => {
	const { command } = params;

	if (command === "pest-language-support.restartServer") {
		ls.free();
		ls = new PestLanguageServer(diagnosticsCallback);
		connection.window.showInformationMessage(
			"Restarted Pest LS; you may need to reopen your Pest files."
		);
	} else {
		log(`Received unknown command: ${command}`);
		connection.window.showWarningMessage(`Unknown command ${command}.`);
	}
});

connection.onInitialize(() => {
	return {
		capabilities: {
			textDocumentSync: {
				openClose: true,
				save: false,
				change: TextDocumentSyncKind.Full,
			},
			renameProvider: true,
			hoverProvider: true,
			declarationProvider: true,
			definitionProvider: true,
			referencesProvider: true,
			completionProvider: {},
			documentFormattingProvider: true,
			workspace: {
				workspaceFolders: { supported: false },
				fileOperations: {
					didDelete: {
						filters: [{ pattern: { glob: "**" } }],
					},
				},
			},
			executeCommandProvider: {
				commands: ["pest-language-support.restartServer"],
			},
		},
	};
});

connection.listen();
