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

  it("uses the packaged app avatar instead of the old placeholder mark", () => {
    const oldSubtitle = ["通用", "桌宠", "运行时"].join("");

    expect(controlAppSource).toContain('const appIconUrl = "/app-icon.png"');
    expect(controlAppSource).toContain('class="mark"');
    expect(controlAppSource).toContain("src={appIconUrl}");
    expect(controlAppSource).not.toContain(oldSubtitle);
  });

  it("labels the PetDex input as a web page link field", () => {
    expect(controlAppSource).toContain('placeholder="PetDex 宠物网页链接"');
    expect(controlAppSource).toContain('status = "请输入 PetDex 宠物网页链接"');
    expect(controlAppSource).not.toContain('placeholder="boba 或 PetDex 链接"');
    expect(controlAppSource).not.toContain("请输入 PetDex ID 或链接");
  });

  it("keeps focus controls out of the desktop control page", () => {
    expect(controlAppSource).not.toContain('class="focus-panel"');
    expect(controlAppSource).not.toContain('class="focus-summary"');
    expect(controlAppSource).not.toContain(">开始专注</button>");
    expect(controlAppSource).not.toContain(">开始休息</button>");
  });

  it("keeps focus action mapping customization visible on the desktop control page", () => {
    expect(controlAppSource).toContain('title: "专注模式"');
    expect(controlAppSource).toContain('item.group === "focus"');
  });

  it("hides Codex interaction controls from the desktop control page", () => {
    expect(controlAppSource).not.toContain('title: "Codex"');
    expect(controlAppSource).not.toContain('item.group === "codex"');
    expect(controlAppSource).not.toContain('class="runtime-status"');
    expect(controlAppSource).not.toContain("<span>Codex</span>");
  });

  it("uses a generic reset label for action mappings", () => {
    expect(controlAppSource).toContain(">恢复默认");
    expect(controlAppSource).not.toContain("恢复 Codex 默认");
  });
});
