### 功能与优化
- 调整工程结构，拆解仓库管理、事件管理、会话管理
- 全面升级 egui、tokio 等依赖库
- sync hard 模式，默认检测并删除 .git 目录下的 shallow.lock 和 index.lock 文件，避免文件锁导致 sync 失败
- gui 集成 tracing 日志，增加 help -> open log 入口
- gui 支持拖拽配置文件后，自动识别并刷新
- gui 打开配置文件自动定位该文件
- gui 隐藏空 labels
