import js from "@eslint/js";
import * as tsParser from "@typescript-eslint/parser";
import solid from "eslint-plugin-solid/configs/typescript";

export default [js.configs.recommended, {
  files: ["**/*.{ts,tsx}"],
  ...solid,
  languageOptions: { parser: tsParser, parserOptions: { project: "./tsconfig.app.json" }, globals: globals.browser },
}];
