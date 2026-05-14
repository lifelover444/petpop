# PetPop

> **推荐下载免安装版： [PetPop release petpop.exe](src-tauri/target/release/petpop.exe)**
>
> 安装版： [PetPop_0.1.0_x64-setup.exe](src-tauri/target/release/bundle/nsis/PetPop_0.1.0_x64-setup.exe)
>
> MSI 安装包： [PetPop_0.1.0_x64_en-US.msi](src-tauri/target/release/bundle/msi/PetPop_0.1.0_x64_en-US.msi)

PetPop 是一个 Windows 优先的 Tauri 2 + Svelte 桌宠运行时，用来导入并运行 Codex 兼容宠物。它会把宠物渲染成透明、置顶、可拖动的桌面 companion，并支持 Hatch Pet、Codex 本地宠物和 PetDex 宠物网页链接导入。

## 功能

- 透明置顶桌宠窗口，支持拖动和位置记忆。
- 控制面板可导入、本地管理和切换宠物。
- 支持 `.zip` 宠物包、宠物文件夹、Codex 本地宠物扫描和 PetDex 导入。
- 支持 Codex 固定 8x9 精灵图契约。
- 支持全局宠物缩放、动作映射和轻量专注计时面板。
- 不会修改 `%USERPROFILE%/.codex/pets` 或仓库示例宠物；导入副本保存在 `%APPDATA%/PetPop/pets/`。

## 下载和运行

推荐下载免安装版，直接运行即可：

1. 下载 [petpop.exe](src-tauri/target/release/petpop.exe)。
2. 双击运行。
3. 启动 PetPop 后，在右侧导入区选择宠物包、扫描 Codex 宠物，或输入 PetDex 宠物网页链接。

如果你希望走安装向导，也可以使用 [PetPop_0.1.0_x64-setup.exe](src-tauri/target/release/bundle/nsis/PetPop_0.1.0_x64-setup.exe)。

## PetDex 导入

在控制面板右侧的 `PetDex` 输入框中粘贴 PetDex 宠物网页链接，然后点击 `导入 PetDex 宠物`。PetPop 会通过 PetDex manifest 解析宠物资源，并把导入副本保存到 `%APPDATA%/PetPop/pets/`。

## 支持的宠物格式

PetPop 保持 Codex 宠物契约兼容：

- 精灵图尺寸：`1536x1872`
- 网格：`8` 列 x `9` 行
- 单格尺寸：`192x208`
- 元数据：`pet.json`，包含 `id`、`displayName`、`description`、`spritesheetPath`

PetPop 专用元数据会写入 `petpop.pet.json`，不会污染 Codex 兼容的 `pet.json`。

## 本地开发

```powershell
npm install
npm run check
npm test
npm run tauri:dev
```

如果 Windows 上已有 `petpop.exe` 正在运行，重新构建前先停止进程：

```powershell
Get-Process petpop -ErrorAction SilentlyContinue | Stop-Process -Force
```

## 打包正式版

```powershell
npm run tauri -- build
```

构建完成后会生成：

- `src-tauri/target/release/petpop.exe`
- `src-tauri/target/release/bundle/nsis/PetPop_0.1.0_x64-setup.exe`
- `src-tauri/target/release/bundle/msi/PetPop_0.1.0_x64_en-US.msi`

## 项目结构

- `src/`：Svelte 前端、控制面板、桌宠窗口和专注面板。
- `src/lib/`：动作映射、动画表、运行时状态和 Tauri 命令封装。
- `src-tauri/`：Tauri/Rust 桌面端、导入校验、PetDex 下载、托盘菜单和持久化数据。
- `public/app-icon.png`：前端头像和 favicon。
- `src-tauri/icons/icon.ico`：Windows 应用、托盘和打包图标。

## 数据位置

- 导入宠物：`%APPDATA%/PetPop/pets/`
- 应用设置：`%APPDATA%/PetPop/settings.json`
- Codex 活动桥：`%APPDATA%/PetPop/codex-activity.json`
