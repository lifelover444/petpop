# PetPop

PetPop 是一个 Windows 优先的轻量桌宠软件，用来运行 Codex 兼容宠物。它支持导入 Hatch Pet 生成的宠物、Codex 本地宠物和 PetDex 上的现成宠物，让 Codex 宠物可以作为通用桌面宠物常驻在桌面上。

## 当前能力

- Tauri 2 + Svelte/Vite 桌面应用。
- 一个控制台面板，用于选择宠物、预览动作、缩放、导入宠物。
- 一个透明、置顶、无边框的常驻宠物窗口。
- 一个透明、置顶、无边框的专注模式气泡窗口，可从桌宠右键打开。
- 系统托盘菜单支持显示控制台、显示/隐藏桌宠和退出应用；关闭控制台会隐藏到托盘。
- 常驻宠物窗口可拖动，会限制到可见屏幕区域内，并会记住上次位置。
- 左右拖动时切换 `running-left` / `running-right` 动作。
- 支持按事件配置动作映射，默认覆盖基础交互、Codex 状态和专注模式；控制台当前暴露基础交互和专注模式映射。
- 支持轻量专注模式，包含专注/休息计时、暂停、继续、完成和取消。
- 支持 Codex 状态桥文件 `%APPDATA%/PetPop/codex-activity.json`，用于把外部任务状态映射到桌宠动作。
- 支持本地 zip、宠物文件夹、Codex pets 扫描和 PetDex ID/URL 导入。
- 导入后的宠物存放在 `%APPDATA%/PetPop/pets/`，不会修改 Codex 原目录。
- 支持从 PetPop 中移除宠物，只删除 `%APPDATA%/PetPop/pets/` 下的导入副本。
- 启动时会迁移旧 `%APPDATA%/PetDesk` 数据到 `%APPDATA%/PetPop`。
- 缩放范围为 `0.1x` 到 `1.0x`，默认 `0.5x`。

## 宠物格式

PetPop 兼容 Codex 宠物包格式：

- `pet.json`
- `spritesheet.webp`
- 图集尺寸：`1536x1872`
- 网格：`8` 列 x `9` 行
- 单格：`192x208`

`pet.json` 至少需要包含：

```json
{
  "id": "taffy",
  "displayName": "Taffy",
  "description": "A Codex-compatible digital pet.",
  "spritesheetPath": "spritesheet.webp"
}
```

PetPop 不改写 Codex 的 `pet.json` 语义。每只导入宠物目录下的 `petpop.pet.json` 记录来源、默认动作映射和导入时间；应用级 `petpop.pet.json` 记录桌宠窗口位置；`settings.json` 记录专注/休息时长和全局宠物缩放。

## 数据目录

默认数据目录是 `%APPDATA%/PetPop/`：

| 路径 | 用途 |
| --- | --- |
| `pets/<pet-id>/` | 用户主动导入的宠物副本 |
| `pets/<pet-id>/pet.json` | Codex 兼容宠物元数据 |
| `pets/<pet-id>/petpop.pet.json` | PetPop 的宠物来源和动作映射 |
| `petpop.pet.json` | PetPop 的应用级元数据，目前用于保存桌宠窗口位置 |
| `settings.json` | 专注/休息默认时长和全局宠物缩放 |
| `codex-activity.json` | 可选 Codex 状态桥输入文件 |
| `cache/` | zip 解压和 PetDex 下载的临时 staging 数据 |

PetPop 会读取 `%USERPROFILE%/.codex/pets` 来导入 Codex 宠物，但不会在正常运行中修改该目录。

## 动作映射

PetPop 复用 Codex 固定 9 行动作，并把不同事件映射到这些动作。动画状态本身保持 Codex 兼容，不新增行：

| 动作 | 场景 |
| --- | --- |
| `idle` | 默认待机 |
| `running-right` | 向右拖动或移动 |
| `running-left` | 向左拖动或移动 |
| `waving` | 问候、唤醒、点击 |
| `jumping` | 双击、成功、完成 |
| `failed` | 错误或导入失败 |
| `waiting` | 长时间无操作 |
| `running` | 任务进行中 |
| `review` | 专注、检查、审阅 |

默认动作事件分为三组：

- 基础交互：拖拽、点击、双击、空闲、成功、失败、审阅。
- Codex：`codex-running`、`codex-waiting`、`codex-review`、`codex-success`、`codex-error`。
- 专注模式：`focus-start`、`focus-pause`、`focus-resume`、`focus-complete`、`focus-cancel`、`break-start`、`break-complete`。

控制台中的“动作映射”区域会自动保存当前宠物的可见映射。内部事件 `drag-start`、`drag-end`、`task-running`、`success`、`error`、`review` 保留为运行时默认映射，当前不作为控制台选项展示。

运行时会按优先级选择当前动作：用户交互和拖拽优先，其次是短反馈、Codex 状态、专注状态，最后才是空闲或等待。

## 专注模式

专注模式是轻量计时器，不做任务管理或统计报表：

- 默认专注 `25` 分钟，休息 `5` 分钟。
- 支持开始、暂停、继续、完成、取消专注。
- 支持手动开始和完成休息。
- 右键桌宠会打开专注模式入口；气泡失焦后自动隐藏。
- 专注气泡支持打开主控制台。
- 专注和休息时长保存到 `%APPDATA%/PetPop/settings.json`。
- 进行中的倒计时不跨重启恢复，重启后回到未开始状态。

## Codex 状态桥

PetPop 会轮询 `%APPDATA%/PetPop/codex-activity.json`，把外部 Codex 或脚本写入的状态映射到桌宠动作。最小格式：

```json
{
  "status": "running",
  "message": "正在执行任务",
  "updatedAt": 1770000000000
}
```

`status` 支持 `idle`、`running`、`waiting`、`review`、`success`、`error`。过期状态会被视为 `idle`；JSON 无效时控制台会显示错误，但不会中断桌宠运行。

## 运行环境

需要安装：

- Node.js 22 或兼容版本
- npm
- Rust stable MSVC toolchain
- Windows WebView2
- Visual Studio C++ Build Tools 或 Visual Studio 2022 MSVC

可用以下命令检查 Tauri 环境：

```powershell
npm run tauri -- info
```

## 开发运行

安装依赖：

```powershell
npm install
```

只运行前端预览：

```powershell
npm run dev
```

运行桌面应用：

```powershell
npm run tauri:dev
```

构建 debug 桌面应用：

```powershell
npm run tauri -- build --debug
```

或使用脚本别名：

```powershell
npm run tauri:build -- --debug
```

Windows 上如果 `petpop.exe` 正在运行，重建前需要先停止进程：

```powershell
Get-Process petpop -ErrorAction SilentlyContinue | Stop-Process -Force
```

## 验证

提交前建议运行：

```powershell
npm run check
npm test
npm run tauri -- build --debug
```

## 目录结构

```text
src/
  App.svelte            根据 Tauri 窗口 label 挂载控制台、桌宠或专注面板
  components/
    ControlApp.svelte   控制台面板
    FocusPanel.svelte   透明专注气泡窗口
    PetWindow.svelte    常驻透明宠物窗口
    SpritePet.svelte    精灵图动画播放器
  lib/
    animations.ts       Codex 动作行表
    actions.ts          动作事件和默认映射
    interactionTiming.ts 点击/双击时序常量
    petInteraction.ts   点击、双击、拖拽方向判定
    petpop.ts           Tauri 命令封装
    runtimeScene.ts     Codex、专注、空闲状态到 scene 事件的转换
    sceneEngine.ts      场景调度基础逻辑
src-tauri/
  src/main.rs           导入、校验、PetDex、托盘、运行状态和配置持久化
  tauri.conf.json       Tauri 三窗口和打包配置
  capabilities/         Tauri 权限配置
taffy/ sakiko/          样例宠物
```

## 文档

- [AGENTS.md](./AGENTS.md)：给后续编码 agent 使用的项目规范。
- [CONSTRAINTS.zh-CN.md](./CONSTRAINTS.zh-CN.md)：中文产品和实现约束。

## 说明

PetDex 宠物是用户提交的 fan art。PetPop 不内置第三方 PetDex 宠物，只提供用户主动导入能力。
