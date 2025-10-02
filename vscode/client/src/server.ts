/* eslint-disable @typescript-eslint/no-unsafe-call */
// eslint-disable-next-line prettier/prettier

/* eslint-disable @typescript-eslint/no-unsafe-argument */

/* eslint-disable @typescript-eslint/no-unsafe-member-access */
import { outputChannel } from ".";
import { exec, ExecException, spawn } from "child_process";
import { stat } from "fs/promises";
import fetch, { AbortError } from "node-fetch";
import path, { join } from "path";
import { promisify } from "util";
import { Progress, ProgressLocation, window, workspace } from "vscode";

export async function findServer(): Promise<string | undefined> {
	const path = await findServerPath();

	if (!path || !(await checkValidity(path))) {
		return undefined;
	}

	outputChannel.appendLine(`[TS] Using server path ${path}.`);

	const config = workspace.getConfiguration("pestIdeTools");
	const updateCheckerEnabled = config.get("checkForUpdates") as boolean;

	// use quotes around path because the path may have spaces in it
	const currentVersion = await promisify(exec)(`"${path}" --version`).then(s =>
		s.stdout.trim(),
	);
	outputChannel.appendLine(`[TS] Server version: v${currentVersion}`);

	if (updateCheckerEnabled && config.get("serverPath") === null) {
		outputChannel.appendLine("[TS] Checking for updates...");

		try {
			const abortController = new AbortController();
			const timeout = setTimeout(() => abortController.abort(), 2500);
			let res;

			try {
				res = await fetch(
					"https://crates.io/api/v1/crates/pest-language-server",
					{ signal: abortController.signal },
				);
				res = await res.json();
			} catch (e) {
				if (e instanceof AbortError) {
					outputChannel.appendLine("[TS] Update check timed out after 2000ms.");
					return path;
				} else {
					throw e;
				}
			} finally {
				clearTimeout(timeout);
			}

			// eslint-disable-next-line @typescript-eslint/ban-ts-comment
			// @ts-ignore
			const latestVersion = res["crate"]["max_version"] as string;

			if (currentVersion !== latestVersion) {
				const choice = await window.showInformationMessage(
					`A new version of the Pest Language Server is available (v${currentVersion} --> v${latestVersion}). Would you like to update automatically?`,
					{},
					"Yes",
				);

				if (choice) {
					if (!(await installBinaryViaCargoInstall())) {
						await window.showErrorMessage(
							"Failed to update Pest Language Server.",
						);
					}
				}
			}
		} catch {
			outputChannel.appendLine("[TS] Failed to run update check.");
		}
	}

	return path;
}

async function findServerPath(): Promise<string | undefined> {
	const config = workspace.getConfiguration("pestIdeTools");

	// Check for custom server path
	if (config.get("serverPath") && workspace.workspaceFolders !== undefined) {
		outputChannel.appendLine(
			path.resolve(
				workspace.workspaceFolders[0].uri.fsPath,
				config.get("serverPath") as string,
			),
		);
		return path.resolve(
			workspace.workspaceFolders[0].uri.fsPath,
			config.get("serverPath") as string,
		);
	}

	const cargoBinDirectory = getCargoBinDirectory();

	if (!cargoBinDirectory) {
		outputChannel.appendLine("[TS] Could not find cargo bin directory.");
		return undefined;
	}

	const expectedPath = join(cargoBinDirectory, getExpectedBinaryName());
	outputChannel.appendLine(`[TS] Trying path ${expectedPath}...`);

	if (await checkValidity(expectedPath)) {
		return expectedPath;
	}

	const choice = await window.showWarningMessage(
		"Failed to find an installed Pest Language Server. Would you like to install one using `cargo install`?",
		{},
		"Yes",
	);

	if (!choice) {
		outputChannel.appendLine("[TS] Not installing server.");
		return undefined;
	}

	if (await installBinaryViaCargoInstall()) {
		return expectedPath;
	} else {
		await window.showErrorMessage(
			"Failed to install Pest Language Server. Please either run `cargo install pest-language-server`, or set a custom path using the configuration `pestIdeTools.serverPath`.",
		);
	}

	return undefined;
}

function getCargoBinDirectory(): string | undefined {
	const cargoInstallRoot = process.env["CARGO_INSTALL_ROOT"];

	if (cargoInstallRoot) {
		return cargoInstallRoot;
	}

	const cargoHome = process.env["CARGO_HOME"];

	if (cargoHome) {
		return join(cargoHome, "bin");
	}

	let home = process.env["HOME"];

	if (process.platform === "win32") {
		home = process.env["USERPROFILE"];
	}

	if (home) {
		return join(home, ".cargo", "bin");
	}

	return undefined;
}

function getExpectedBinaryName(): string {
	switch (process.platform) {
		case "win32":
			return "pest-language-server.exe";
		default:
			return "pest-language-server";
	}
}

async function checkValidity(path: string): Promise<boolean> {
	try {
		await stat(path);
		return true;
	} catch {
		return false;
	}
}

async function installBinaryViaCargoInstall(): Promise<boolean> {
	outputChannel.appendLine("[TS] Installing server.");

	return await window.withProgress(
		{
			location: ProgressLocation.Notification,
			cancellable: true,
			title: "Installing Pest Language Server",
		},
		async (progress, token) => {
			try {
				progress.report({ message: "spawning `cargo install` command" });

				const process = spawn("cargo", ["install", "pest-language-server"], {
					shell: true,
				});

				token.onCancellationRequested(() => {
					process.kill();
				});

				process.stderr.on("data", data =>
					logCargoInstallProgress(data.toString(), progress),
				);

				process.stdout.on("data", data =>
					logCargoInstallProgress(data.toString(), progress),
				);

				const exitCode: number = await new Promise((resolve, _) => {
					process.on("close", resolve);
				});

				outputChannel.appendLine(
					`[TS]: Cargo process exited with code ${exitCode}`,
				);

				if (exitCode === 0) {
					progress.report({ increment: 100, message: "installed" });

					return true;
				} else {
					throw new Error(`Received non-zero exit code: ${exitCode}`);
				}
			} catch (e) {
				outputChannel.appendLine(`[TS] ${(e as ExecException).message}`);
				progress.report({ message: "An error occurred." });
				return false;
			}
		},
	);
}

function logCargoInstallProgress(
	data: string,
	progress: Progress<{
		message?: string | undefined;
		increment?: number | undefined;
	}>,
) {
	data = data.trim();

	let msg;
	const versionRegex = /v[0-9]+\.[0-9]+\.[0-9]+/;

	if (data === "Updating crates.io index") {
		msg = "updating crates.io index";
	} else if (
		data ===
		"Updating git repository `https://github.com/pest-parser/pest-ide-tools`"
	) {
		msg = "fetching crate";
	} else if (data.startsWith("Compiling")) {
		msg = `compiling ${data.split("Compiling ")[1].split(versionRegex)[0]}`;
	}

	if (msg) {
		outputChannel.appendLine(`[TS] \t${msg}`);
		progress.report({ message: `\t${msg}` });
	}
}
