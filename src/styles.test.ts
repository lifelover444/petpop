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
});

describe("focus panel layout", () => {
  it("keeps focus settings visually subordinate to the timer status", () => {
    const focusRule = styles.match(/\.focus-panel\s*\{(?<body>[^}]*)\}/)
      ?.groups?.body;
    const summaryRule = styles.match(/\.focus-summary\s*\{(?<body>[^}]*)\}/)
      ?.groups?.body;

    expect(focusRule).toBeDefined();
    expect(focusRule).toContain("grid-template-columns");
    expect(summaryRule).toBeDefined();
    expect(summaryRule).toContain("grid-template-columns");
  });
});
