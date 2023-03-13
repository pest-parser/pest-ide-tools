import { outputChannel } from ".";
import { exec, ExecException } from "child_process";
import { stat } from "fs/promises";
import fetch, { Response } from "node-fetch";
import { join } from "path";
import { promisify } from "util";
import { window, workspace } from "vscode";

export async function findServer(): Promise<string | undefined> {
	const path = await findServerPath();

	if (!path || !(await checkValidity(path))) {
		return undefined;
	}

	outputChannel.appendLine(`[TS] Using server path ${path}.`);

	const config = workspace.getConfiguration("pestIdeTools");
	const updateCheckerEnabled = config.get("checkForUpdates") as boolean;

	if (updateCheckerEnabled) {
		try {
			const { stdout: currentVersion } = await promisify(exec)(
				`${path} --version`
			);
			outputChannel.appendLine(
				`[TS] Server version: v${currentVersion.trimEnd()}`
			);

			const res = (await Promise.race([
				fetch("https://crates.io/api/v1/crates/pest_language_server"),
				new Promise(() => {
					setTimeout(() => {
						throw new Error("Timed out.");
					}, 2000);
				}),
			])) as Response;

			// eslint-disable-next-line @typescript-eslint/ban-ts-comment
			// @ts-ignore
			// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
			const latestVersion = (await res.json())["crate"][
				"max_version"
			] as string;

			if (currentVersion !== latestVersion) {
				const choice = await window.showInformationMessage(
					`A new version of the Pest Language Server is available (v${currentVersion} -> v${latestVersion}). Would you like to update automatically?`,
					{},
					"Yes"
				);

				if (choice) {
					if (!(await installBinaryViaCargoInstall())) {
						await window.showErrorMessage(
							"Failed to update Pest Language Server."
						);
					}
				}
			}
		} catch (_) {
			outputChannel.appendLine("[TS] Failed to run update check.");
		}
	}

	return path;
}

async function findServerPath(): Promise<string | undefined> {
	const config = workspace.getConfiguration("pestIdeTools");

	// Check for custom server path
	if (config.has("serverPath")) {
		return config.get("serverPath") as string;
	}

	const cargoInstallRoot = getCargoInstallRoot();

	if (!cargoInstallRoot) {
		outputChannel.appendLine("[TS] Could not find cargo bin directory.");
		return undefined;
	}

	const expectedPath = join(cargoInstallRoot, getExpectedBinaryName());

	if (await checkValidity(expectedPath)) {
		return expectedPath;
	}

	const choice = await window.showWarningMessage(
		"Failed to find an installed Pest Language Server. Would you like to install one using `cargo install`?",
		{},
		"Yes"
	);

	if (!choice) {
		return undefined;
	}

	if (await installBinaryViaCargoInstall()) {
		return expectedPath;
	} else {
		await window.showErrorMessage(
			"Failed to install Pest Language Server. Please either run `cargo install pest-language-server`, or set a custom path using the configuration `pestIdeTools.serverPath`."
		);
	}

	return undefined;
}

function getCargoInstallRoot(): string | undefined {
	const cargoInstallRoot = process.env["CARGO_INSTALL_ROOT"];

	if (cargoInstallRoot) {
		return cargoInstallRoot;
	}

	const cargoHome = process.env["CARGO_HOME"];

	if (cargoHome) {
		return join(cargoHome, "bin");
	}

	const home = process.env["HOME"];

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
	} catch (_) {
		return false;
	}
}

async function installBinaryViaCargoInstall(): Promise<boolean> {
	try {
		await promisify(exec)("cargo install pest-language-server");
		return true;
	} catch (e) {
		outputChannel.appendLine(`[TS] ${(e as ExecException).message}`);
		return false;
	}
}
