import recommendedIncremental from "eslint-config-agent/recommended-incremental";
import globals from "globals";

/**
 * Incremental adoption of eslint-config-agent.
 *
 * This package previously had no ESLint setup at all, so `src/index.js` was
 * never linted. Starting on the `recommended-incremental` preset (the divisive
 * strict-mode rules disabled, everything else downgraded to a warning) keeps
 * `pnpm lint` exiting 0 while still surfacing the full backlog. Promote to
 * `eslint-config-agent/recommended` — or the strict default export — once the
 * warnings are burned down.
 */
export default [
  { ignores: ["node_modules/**"] },
  ...recommendedIncremental,
  {
    // A Node.js HTTP server, not browser code: `process`, `Buffer` and
    // `console` are runtime globals here, and the default browser globals the
    // shared config assumes would otherwise leave every one of them
    // undefined under `no-undef`.
    files: ["**/*.js"],
    languageOptions: {
      sourceType: "module",
      globals: {
        ...globals.node,
      },
    },
    // The shared config leaves eslint-plugin-react's version on "detect".
    // There is no React here (this is a server with zero runtime deps), so
    // detection fails and the plugin prints a warning on every run. Pin a
    // version to keep the lint output clean; no React rules actually apply.
    settings: {
      react: { version: "19.0.0" },
    },
  },
];
