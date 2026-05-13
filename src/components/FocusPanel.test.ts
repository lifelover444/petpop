import { describe, expect, it } from "vitest";
import focusPanelSource from "./FocusPanel.svelte?raw";

describe("focus panel styles", () => {
  it("does not globally disable scrolling for every app window", () => {
    expect(focusPanelSource).not.toContain(":global(html),");
    expect(focusPanelSource).not.toContain(":global(body),");
    expect(focusPanelSource).not.toContain(":global(#app)");
    expect(focusPanelSource).toContain("focus-panel-runtime");
  });

  it("renders as a compact timer capsule", () => {
    expect(focusPanelSource).toContain('class="focus-capsule"');
    expect(focusPanelSource).toContain('class="capsule-meta"');
    expect(focusPanelSource).toContain('class="capsule-settings"');
    expect(focusPanelSource).toContain('class="capsule-progress"');
    expect(focusPanelSource).toContain('class="capsule-actions"');
  });

  it("starts as a small explicit focus launcher before showing the timer capsule", () => {
    expect(focusPanelSource).toContain("let capsuleOpen = $state(false)");
    expect(focusPanelSource).toContain("const FOCUS_LAUNCHER_WIDTH = 132");
    expect(focusPanelSource).toContain("const FOCUS_LAUNCHER_HEIGHT = 46");
    expect(focusPanelSource).toContain('class="focus-launcher-surface"');
    expect(focusPanelSource).toContain('class="focus-launcher"');
    expect(focusPanelSource).toContain(">打开专注模式</button>");
    expect(focusPanelSource).toContain("onclick={openCapsule}");
    expect(focusPanelSource).toContain("width: 132px");
    expect(focusPanelSource).toContain("height: 46px");
  });

  it("resets to the launcher when the Tauri window is shown at launcher size", () => {
    expect(focusPanelSource).toContain("syncCapsuleModeToWindowSize");
    expect(focusPanelSource).toContain("onResized");
    expect(focusPanelSource).toContain("onFocusChanged");
    expect(focusPanelSource).toContain("currentWindow.scaleFactor()");
    expect(focusPanelSource).toContain("currentSize.width <= FOCUS_LAUNCHER_WIDTH * scaleFactor + 1");
    expect(focusPanelSource).toContain("capsuleOpen = false");
  });

  it("uses the close button only to hide the bubble", () => {
    expect(focusPanelSource).toContain('class="close-button"');
    expect(focusPanelSource).toContain('aria-label="关闭气泡"');
    expect(focusPanelSource).toContain("onclick={closePanel}");
  });

  it("uses the more button to open the desktop panel", () => {
    expect(focusPanelSource).toContain('class="icon-button more-icon"');
    expect(focusPanelSource).toContain('aria-label="打开桌面端"');
    expect(focusPanelSource).toContain("onclick={openMainPanel}");
    expect(focusPanelSource).toContain("showMainWindow");
    expect(focusPanelSource).toContain("await showMainWindow()");
  });

  it("allows focus duration edits inside the bubble", () => {
    expect(focusPanelSource).toContain("setAppSettings");
    expect(focusPanelSource).toContain("updateFocusMinutes");
    expect(focusPanelSource).toContain('class="duration-stepper"');
    expect(focusPanelSource).toContain('aria-label="专注时长"');
  });

  it("renders reset as a clear secondary text action", () => {
    expect(focusPanelSource).toContain('class="reset-button"');
    expect(focusPanelSource).toContain(">重置</button>");
    expect(focusPanelSource).not.toContain("reset-icon::before");
    expect(focusPanelSource).not.toContain("reset-icon::after");
  });

  it("uses compact timer copy without explanatory instructions", () => {
    expect(focusPanelSource).toContain("formatPanelTime");
    expect(focusPanelSource).toContain("primaryAction");
    expect(focusPanelSource).not.toContain("点击圆形按钮");
    expect(focusPanelSource).not.toContain("开始后会在这里显示计时状态");
  });

  it("keeps the timer text clean and makes progress represent remaining time", () => {
    expect(focusPanelSource).not.toContain("text-overflow: ellipsis");
    expect(focusPanelSource).toContain("return 1;");
    expect(focusPanelSource).toContain("remainingMs / totalMs");
    expect(focusPanelSource).not.toContain("1 - remainingMs / totalMs");
  });

  it("avoids an outer square frame around the rounded capsule", () => {
    expect(focusPanelSource).toContain("box-shadow: none");
    expect(focusPanelSource).toContain("inset 0 0 0 1px");
    expect(focusPanelSource).not.toContain("0 14px 32px");
  });

  it("keeps the focus capsule small enough to sit beside the pet", () => {
    expect(focusPanelSource).toContain("width: 100vw");
    expect(focusPanelSource).toContain("height: 100vh");
    expect(focusPanelSource).toContain("display: grid");
  });
});
