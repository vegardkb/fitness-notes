import js from "@eslint/js";
import svelte from "eslint-plugin-svelte";
import globals from "globals";
import ts from "typescript-eslint";

export default ts.config(
    {
        ignores: [
            "build/",
            ".svelte-kit/",
            "dist/",
            "src-tauri/gen/",
            "src-tauri/target/",
            "src/lib/licenses.ts",
        ],
    },
    js.configs.recommended,
    ...ts.configs.recommended,
    ...svelte.configs["flat/recommended"],
    {
        files: ["**/*.svelte", "**/*.ts"],
        languageOptions: {
            globals: { ...globals.browser },
        },
        rules: {
            "@typescript-eslint/no-unused-vars": [
                "error",
                { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
            ],
            "@typescript-eslint/no-explicit-any": "warn",
            "svelte/no-navigation-without-resolve": "off",
            "svelte/require-each-key": "off",
        },
    },
    {
        files: ["**/*.svelte"],
        languageOptions: {
            parserOptions: {
                parser: ts.parser,
                extraFileExtensions: [".svelte"],
            },
        },
    },
    {
        files: [
            "scripts/**/*.mjs",
            "*.config.js",
            "*.config.ts",
            "eslint.config.js",
        ],
        languageOptions: {
            globals: { ...globals.node },
        },
    },
);
