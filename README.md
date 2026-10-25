# 云运动助手（Tauri v2）

云运动跑步辅助桌面端，前端 Vite + React + TypeScript，后端 Tauri v2（Rust），国密（SM2/SM4）与云端接口交互。

## 目录结构

```
yunsport-assistant/
├── index.html / vite.config.ts / tsconfig*.json   # 前端工程（Vite+React+TS）
├── public/                                        # 前端静态资源
├── package.json                                   # 顶层脚本：dev / tauri dev / tauri build
├── src/                                           # 前端源码
│   ├── main.tsx / App.tsx                         # 入口与壳层（标题栏 + Tab）
│   ├── components/                                # 通用展示组件
│   ├── pages/                                     # 页面（login/home/sim/draw/detail/set）
│   ├── hooks/                                     # Tauri 运行时 hook（类型化 invoke / 事件监听）
│   ├── services/                                  # invoke 命令层，按 Rust commands/ 域拆分
│   ├── store/                                     # 全局会话与配置（AppProvider）
│   ├── types/                                     # 与 Rust 结构体对齐的 TS 类型
│   └── utils/                                     # 地图/时间等纯工具
└── src-tauri/                                     # Rust 后端（Cargo 包）
    ├── Cargo.toml / build.rs / tauri.conf.json
    ├── capabilities/default.json                  # Tauri v2 权限
    ├── icons/
    └── src/
        ├── main.rs                                # 极简入口 → yunsport::run()
        ├── lib.rs                                 # 装配：命令注册/状态/启动任务
        ├── commands/                              # invoke 命令（按域拆分，薄壳）
        ├── services/                              # 业务服务层（纯 Rust，可测）
        ├── models/                                # 数据模型（serde）
        ├── state/                                 # Tauri 托管状态
        ├── utils/                                 # crypto/protocol/路径等工具
        └── error.rs                               # 统一错误类型
```

## 开发

```bash
pnpm install
pnpm tauri dev
```

## 构建

```bash
pnpm tauri build
```
