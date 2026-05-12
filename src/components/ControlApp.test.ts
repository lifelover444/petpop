import { describe, expect, it } from "vitest";
import controlAppSource from "./ControlApp.svelte?raw";

describe("control app source", () => {
  it("does not reconcile the global pet scene during routine refresh", () => {
    const refreshFunction = controlAppSource.match(
      /async function refresh\(\) \{[\s\S]*?\n  \}/,
    )?.[0];

    expect(refreshFunction).toBeTruthy();
    expect(refreshFunction).not.toContain("reconcileScene");
  });

  it("confirms before removing the active pet copy", () => {
    const removeFunction = controlAppSource.match(
      /async function removeActivePet\(\) \{[\s\S]*?\n  \}/,
    )?.[0];

    expect(removeFunction).toBeTruthy();
    expect(removeFunction).toContain("window.confirm");
    expect(removeFunction).toContain("removePet(activePet.id)");
  });

  it("renders a remove button for the active pet", () => {
    expect(controlAppSource).toContain("移除宠物");
    expect(controlAppSource).toContain("onclick={removeActivePet}");
  });

  it("organizes focus mode as status, settings, and actions", () => {
    expect(controlAppSource).toContain('class="focus-summary"');
    expect(controlAppSource).toContain('class="focus-settings"');
    expect(controlAppSource).toContain('class="focus-actions"');
    expect(controlAppSource).toContain('class="duration-field"');
  });
});
