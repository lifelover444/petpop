# PetDesk

PetDesk 是一个 Windows 优先的轻量桌宠软件，用来运行 Codex 兼容宠物。它支持导入 Hatch Pet 生成的宠物、Codex 本地宠物和 PetDex 上的现成宠物，让 Codex 宠物可以作为通用桌面宠物常驻在桌面上。

## 当前能力

- Tauri 2 + Svelte/Vite 桌面应用。
- 一个控制台面板，用于选择宠物、预览动作、缩放、导入宠物。
- 一个透明、置顶、无边框的常驻宠物窗口。
- 常驻宠物窗口可拖动，并会记住上次位置。
- 左右拖动时切换 `running-left` / `running-right` 动作。
- 支持本地 zip、宠物文件夹、Codex pets 扫描和 PetDex ID/URL 导入。
- 导入后的宠物存放在 `%APPDATA%/PetDesk/pets/`，不会修改 Codex 原目录。

## 宠物格式

PetDesk 兼容 Codex 宠物包格式：

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

PetDesk 自己的来源、缩放、位置等扩展信息会写入 `petdesk.pet.json`，不会改写 Codex 的 `pet.json` 语义。

## 动作映射

PetDesk 复用 Codex 固定 9 行动作，并映射到通用桌面场景：

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

Windows 上如果 `petdesk.exe` 正在运行，重建前需要先停止进程：

```powershell
Get-Process petdesk -ErrorAction SilentlyContinue | Stop-Process -Force
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
  components/
    ControlApp.svelte   控制台面板
    PetWindow.svelte    常驻透明宠物窗口
    SpritePet.svelte    精灵图动画播放器
  lib/
    animations.ts       Codex 动作行表
    petdesk.ts          Tauri 命令封装
    sceneEngine.ts      场景调度基础逻辑
src-tauri/
  src/main.rs           导入、校验、PetDex、运行状态和配置持久化
  tauri.conf.json       Tauri 窗口和打包配置
  capabilities/         Tauri 权限配置
taffy/ sakiko/          样例宠物
```

## 文档

- [AGENTS.md](./AGENTS.md)：给后续编码 agent 使用的项目规范。
- [CONSTRAINTS.zh-CN.md](./CONSTRAINTS.zh-CN.md)：中文产品和实现约束。

## 说明

PetDex 宠物是用户提交的 fan art。PetDesk 不内置第三方 PetDex 宠物，只提供用户主动导入能力。
