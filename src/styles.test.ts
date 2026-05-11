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
