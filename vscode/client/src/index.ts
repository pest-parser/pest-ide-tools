/* eslint-disable @typescript-eslint/restrict-template-expressions */

import { join } from "path";

import {
	ExtensionContext,
	RelativePattern,
	TextDocument,
	Uri,
	window,
	workspace,
	WorkspaceFolder,
	WorkspaceFoldersChangeEvent,
} from "vscode";
import {
	LanguageClient,
	LanguageClientOptions,
	TransportKind,
} from "vscode-languageclient/node";

const extensionName = "Pest Language Server";
const outputChannel = window.createOutputChannel(extensionName);

const clients: Map<string, LanguageClient> = new Map();

function pestFilesInFolderPattern(folder: Uri) {
	return new RelativePattern(folder, "**/*.pest");
}

async function openPestFilesInFolder(folder: Uri) {
	const pattern = pestFilesInFolderPattern(folder);
	const uris = await workspace.findFiles(pattern);
	return Promise.all(uris.map(openDocument));
}

async function openDocument(uri: Uri) {
	const uriMatch = (d: TextDocument) => d.uri.toString() === uri.toString();
	const doc = workspace.textDocuments.find(uriMatch);
	if (doc === undefined) await workspace.openTextDocument(uri);
	return uri;
}

async function startClients(folder: WorkspaceFolder, ctx: ExtensionContext) {
	const server = ctx.asAbsolutePath(join("build", "server.js"));
	const root = folder.uri;

	const pestFilesIncluded: Set<string> = new Set();

	const deleteWatcher = workspace.createFileSystemWatcher(
		pestFilesInFolderPattern(root),
		true, // ignoreCreateEvents
		true, // ignoreChangeEvents
		false // ignoreDeleteEvents
	);

	const createChangeWatcher = workspace.createFileSystemWatcher(
		pestFilesInFolderPattern(root),
		false, // ignoreCreateEvents
		false, // ignoreChangeEvents
		true // ignoreDeleteEvents
	);

	ctx.subscriptions.push(deleteWatcher);
	ctx.subscriptions.push(createChangeWatcher);

	const serverOpts = { module: server, transport: TransportKind.ipc };
	const clientOpts: LanguageClientOptions = {
		documentSelector: [
			{ language: "pest", pattern: `${root.fsPath}/**/*.pest` },
		],
		synchronize: { fileEvents: deleteWatcher },
		diagnosticCollectionName: extensionName,
		workspaceFolder: folder,
		outputChannel,
	};
	const client = new LanguageClient(extensionName, serverOpts, clientOpts);

	ctx.subscriptions.push(client.start());

	ctx.subscriptions.push(createChangeWatcher.onDidCreate(openDocument));
	ctx.subscriptions.push(createChangeWatcher.onDidChange(openDocument));

	const openedFiles = await openPestFilesInFolder(root);
	openedFiles.forEach(f => pestFilesIncluded.add(f.toString()));
	clients.set(root.toString(), client);
}

function stopClient(client: LanguageClient) {
	client.diagnostics?.clear();
	return client.stop();
}

async function stopClients(workspaceFolder: string) {
	const client = clients.get(workspaceFolder);
	if (client) {
		await stopClient(client);
	}

	clients.delete(workspaceFolder);
}

function updateClients(context: ExtensionContext) {
	return async function ({ added, removed }: WorkspaceFoldersChangeEvent) {
		for (const folder of removed) await stopClients(folder.uri.toString());
		for (const folder of added) await startClients(folder, context);
	};
}

export async function activate(context: ExtensionContext): Promise<void> {
	const folders = workspace.workspaceFolders || [];

	for (const folder of folders) await startClients(folder, context);

	workspace.onDidChangeWorkspaceFolders(updateClients(context));
}

export async function deactivate(): Promise<void> {
	await Promise.all([...clients.values()].map(stopClient));
}
