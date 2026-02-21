const gts = require("gts");
const { defineConfig } = require("eslint/config");

module.exports = defineConfig([
	...gts,
	{
		rules: {
			quotes: ["error", "double"],
			indent: ["off"],
		},
	},
	{
		files: ["**/*.ts"],
		languageOptions: {
			parserOptions: {
				project: "./client/tsconfig.json",
			},
		},
		rules: {
			"@typescript-eslint/no-unused-vars": [
				"error",
				{ argsIgnorePattern: "_" },
			],
		},
	},
]);
