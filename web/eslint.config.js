import js from "@eslint/js";
import * as tsParser from "@typescript-eslint/parser";
import solid from "eslint-plugin-solid/configs/typescript";
import globals from "globals";

export default [js.configs.recommended, {
  files: ["**/*.{ts,tsx}"],
  ...solid,
  languageOptions: { parser: tsParser, globals: globals.browser },
}];
