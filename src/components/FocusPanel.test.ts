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
    expect(focusPanelSource).toContain('class="capsule-progress"');
    expect(focusPanelSource).toContain('class="capsule-actions"');
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

  it("keeps the focus capsule small enough to sit beside the pet", () => {
    expect(focusPanelSource).toContain("width: 396px");
    expect(focusPanelSource).toContain("height: 116px");
  });
});
