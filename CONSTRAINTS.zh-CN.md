# PetPop 中文约束文档

## 产品定位

PetPop 是一个 Windows 优先的轻量桌宠运行时，用来运行 Codex 兼容宠物。它把 Hatch Pet 的“一句话生成宠物”、Codex 自定义宠物包、PetDex 现成宠物图库和通用桌面桌宠体验连接起来。

第一版目标不是重新发明宠物格式，而是稳定复用 Codex 宠物包：

- `pet.json`
- `spritesheet.webp`
- 固定 `1536x1872` 图集
- `8 x 9` 网格
- 单格 `192x208`

## 技术约束

- 桌面端使用 Tauri 2，不迁移到 Electron。
- 前端使用 Svelte/Vite，保持界面轻量。
- Rust 后端负责文件导入、解压、校验、PetDex 下载、配置持久化和窗口状态。
- Windows 是 MVP 主平台；跨平台适配不能破坏 Windows 体验。
- 不引入大型状态管理、UI 框架或重型动画库，除非有明确收益。

## 数据和文件约束

- 用户导入的宠物存放在 `%APPDATA%/PetPop/pets/`。
- 扫描 Codex 宠物时只读取 `%USERPROFILE%/.codex/pets`，不要修改它。
- 不能改写 Codex 原生 `pet.json` 语义。
- PetPop 自己的扩展信息写入 `petpop.pet.json`。
- 全局应用设置写入 `%APPDATA%/PetPop/settings.json`，目前用于保存专注和休息默认时长。
- Codex 状态桥文件为 `%APPDATA%/PetPop/codex-activity.json`，只读轮询，不解析 `~/.codex` 日志或 SQLite。
- 不把 PetDex 第三方宠物预置进应用包。用户必须主动导入。
- 仓库中的 `taffy`、`sakiko` 是样例宠物，不应在运行时被修改。

## 宠物包校验约束

导入时必须校验：

- 目录或 zip 内存在 `pet.json`。
- `pet.json` 至少包含 `id`、`displayName`、`spritesheetPath`。
- spritesheet 文件存在并可解码。
- spritesheet 尺寸必须是 `1536x1872`。

运行时列出宠物不能反复解码大图，避免 Debug 版本卡死。完整图片校验只应发生在导入阶段。

## 窗口和交互约束

- 控制台窗口是普通桌面面板。
- 常驻宠物窗口必须：
  - 无边框
  - 透明背景
  - 置顶
  - 不显示在任务栏
  - 可拖动
  - 记住上次位置
- 常驻宠物窗口不能有灰色或浅色底板。
- 拖动宠物时必须按方向切换动作：
  - 向左拖动：`running-left`
  - 向右拖动：`running-right`
  - 松手：`idle`
- 双击宠物触发 `jumping`。
- 开始拖动或唤醒可以触发 `waving`。
- 缩放范围固定为 `0.1x` 到 `1.0x`，默认 `0.5x`。前端控件、浏览器 fallback、Rust clamp 和读取旧 metadata 时都要保持一致。

## 动作映射约束

Codex 的动作状态需要映射到通用桌面场景：

| Codex 状态 | 通用场景 |
| --- | --- |
| `idle` | 默认待机 |
| `waiting` | 长时间无操作或休息 |
| `running` | 任务进行中 |
| `review` | 专注、检查、审阅 |
| `failed` | 错误、失败、导入失败 |
| `waving` | 问候、唤醒、点击 |
| `jumping` | 成功、完成、双击互动 |
| `running-left` | 向左移动 |
| `running-right` | 向右移动 |

不要随意新增或改名状态；如果必须变更，要同步更新动画表、UI、运行时状态和测试。

PetPop 可以新增动作事件，但不能新增 Codex 动画状态。动作事件需要同时更新：

- 前端 `src/lib/actions.ts`
- Rust `PET_ACTION_EVENTS` 和 `default_action_map`
- 动作映射 UI 分组
- 相关单元测试

当前动作事件分为：

- 基础交互：拖拽、点击、双击、空闲、等待、成功、失败、审阅。
- Codex：`codex-running`、`codex-waiting`、`codex-review`、`codex-success`、`codex-error`。
- 专注模式：`focus-start`、`focus-pause`、`focus-resume`、`focus-complete`、`focus-cancel`、`break-start`、`break-complete`。

运行时场景仲裁优先级为：用户交互和拖拽 > 短反馈 > Codex 状态 > 专注模式 > 空闲/等待。

## 专注模式约束

- 专注模式是轻量计时器，不做任务列表、日报、长休息或自动循环。
- 默认专注 `25` 分钟，休息 `5` 分钟。
- 用户手动开始专注、开始休息、暂停、继续、完成或取消。
- 保存默认时长，不保存进行中的倒计时；重启后回到未开始状态。
- 专注模式必须复用动作映射，不直接硬编码动画状态。

## Codex 状态桥约束

- 状态桥文件路径是 `%APPDATA%/PetPop/codex-activity.json`。
- 支持状态：`idle`、`running`、`waiting`、`review`、`success`、`error`。
- 最小 JSON：

```json
{
  "status": "running",
  "message": "正在执行任务",
  "updatedAt": 1770000000000
}
```

- `updatedAt` 可以是秒或毫秒时间戳；读取时统一成毫秒。
- 长时间未更新的运行态应视为过期，不能永久占用桌宠动作。
- JSON 无效时保留上一次有效状态，并在控制台显示错误。

## PetDex 约束

- PetDex 导入优先使用 `https://petdex.crafter.run/api/manifest`。
- 通过 manifest 中的 `petJsonUrl` 和 `spritesheetUrl` 下载资源。
- PetDex 宠物是用户提交的 fan art，界面和文档需要保留来源说明。
- 如果下载失败，应提示用户可以先用 `npx petdex install <id>` 安装到 Codex，再用扫描 Codex 宠物导入。

## UI 约束

- 首屏必须是可用工具界面，不做营销落地页。
- 控制台应该保持安静、实用、信息密度适中。
- 重要操作包括：选择宠物、预览动作、调节缩放、导入本地 zip、导入本地文件夹、导入 PetDex、扫描 Codex pets。
- 常驻宠物窗口只显示宠物，不显示说明、按钮或装饰面板。
- 文案默认使用中文，保留必要的英文状态名和命令名。

## 验证约束

每次改动至少运行：

```powershell
npm run check
npm test
```

涉及 Tauri、Rust、窗口、权限、导入、打包时，还要运行：

```powershell
npm run tauri -- build --debug
```

Windows 上重建前如果 `petpop.exe` 正在运行，先停止：

```powershell
Get-Process petpop -ErrorAction SilentlyContinue | Stop-Process -Force
```

## 提交约束

- 不提交 `node_modules/`、`dist/`、`src-tauri/target/`、`src-tauri/gen/`。
- 需要提交 `src-tauri/Cargo.lock`。
- 未跟踪的宠物 zip 默认视为用户素材，不要擅自提交。
- 提交信息应简洁描述行为变化。
