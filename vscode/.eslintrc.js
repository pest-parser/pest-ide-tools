module.exports = {
	root: true,
	extends: ["./node_modules/gts/"],
	ignorePatterns: ["node_modules", "build"],
	rules: {
		quotes: ["error", "double"],
		indent: ["off"],
		"@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "_" }],
	},
	overrides: [
		{
			files: ["**/*.ts"],
			extends: [
				"plugin:@typescript-eslint/recommended",
				"plugin:@typescript-eslint/recommended-requiring-type-checking",
			],
			parserOptions: {
				tsconfigRootDir: __dirname,
				project: ["./client/tsconfig.json", "./test/tsconfig.json"],
			},
		},
	],
};
