# Pest IDE Tools Documentation

This document contains instructions for setting up Pest support for all of the supported editors.

## Contents

- [Server Configuration](#config)
- [VSCode](#vscode)
- [Sublime Text](#sublime-text)

## Config

The method of updating your config is editor specific.

The available options for all editors are:

```jsonc
{
  // Check for updates to the Pest LS binary via crates.io
  "pestIdeTools.checkForUpdates": true,
  // Ignore specific rule names for the unused rules diagnostics (useful for specifying root rules)
  "pestIdeTools.alwaysUsedRuleNames": [
    "rule_one",
    "rule_two"
  ],
  // Enables logging of performance marks (for developers)
  "pestIdeTools.enablePerformanceLogging": false
}
```

## VSCode

1. Download [the extension](https://marketplace.visualstudio.com/items?itemName=pest.pest-ide-tools).
2. Await the prompt which will ask you if you want to install a suitable binary, and accept.
3. Wait for it to install the server.
    - If the server fails to install, you can install it manually using `cargo install pest-language-server`, then use the configuration `pestIdeTools.serverPath` to point the extension to the installed binary.
4. (_Optional_) You may need to execute the command `Pest: Restart server` or reload your window for the server to activate.

### VSCode Specific Configs

These config options are specific to VSCode.

```jsonc
{
  // Set a custom path to a Pest LS binary
  "pestIdeTools.serverPath": "/path/to/binary",
  // Custom arguments to pass to the Pest LS binary
  "pestIdeTools.customArgs": []
}
```

## Sublime Text

1. Download the `pest.sublime-package` file from the [latest release](https://github.com/pest-parser/pest-ide-tools/releases/latest)'s assets page.
    - This gives you syntax highlighting for Pest flies.
2. Place the downloaded `pest.sublime-package` file in the `path/to/sublime-text/Installed Packages` directory.
3. Install the server using `cargo install pest-language-server`.
3. Execute the `Preferences: LSP Settings` command and add a new key to the `clients` object. 
    ```json
    // LSP.sublime-settings
    "clients": {
		"pest": {
			"enabled": true,
            // This is usually something like /home/username/.cargo/bin/pest-language-server
			"command": ["/path/to/language/server/binary"],
			"selector": "source.pest",
		},
        // ...other LSPs
	}
    ```
4. You may have to restart your Sublime Text to get the LSP to start.
