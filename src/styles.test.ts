import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";

const styles = readFileSync(new URL("./styles.css", import.meta.url), "utf-8");

describe("main window scrolling", () => {
  it("lets the document own vertical scrolling for the control window", () => {
    const shellRule = styles.match(/\.app-shell\s*\{(?<body>[^}]*)\}/)?.groups
      ?.body;

    expect(shellRule).toBeDefined();
    expect(shellRule).toMatch(/(?:^|[;\s])min-height\s*:\s*100vh/);
    expect(shellRule).not.toMatch(/(?:^|[;\s])height\s*:\s*100vh/);
    expect(shellRule).not.toMatch(/(?:^|[;\s])overflow-y\s*:\s*auto/);
  });

  it("keeps the importer from covering the workspace at the minimum width", () => {
    const shellRule = styles.match(/\.app-shell\s*\{(?<body>[^}]*)\}/)?.groups
      ?.body;
    const importerRule = styles.match(/\.importer\s*\{(?<body>[^}]*)\}/)
      ?.groups?.body;

    expect(shellRule).toBeDefined();
    expect(shellRule).toContain("minmax(0, 1fr)");
    expect(shellRule).toContain("260px");
    expect(importerRule).toBeDefined();
    expect(importerRule).toContain("min-width: 0");
  });
});

describe("desktop control page chrome", () => {
  it("does not ship desktop focus panel styles after focus moves to the pet bubble", () => {
    expect(styles).not.toContain(".focus-panel");
    expect(styles).not.toContain(".focus-summary");
    expect(styles).not.toContain(".focus-actions");
  });

  it("does not reserve layout space for Codex activity status on the desktop page", () => {
    expect(styles).not.toContain(".runtime-status");
  });
});
