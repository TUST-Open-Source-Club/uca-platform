# Plan: VolunteerHours (MVP foundation)

## Current Task: UI validation & auth flow + PDF layout
1) 前端表单校验与状态统一
- 增加统一的请求状态与错误提示封装（useRequest）。
- 为登录/二次验证/学生提交/审核/管理/导出等表单补齐校验规则。
- 统一 API 请求中的加载态、错误提示与结果提示。

2) 端到端登录流程与角色保护
- 前端实现 Passkey 登录结束流程（WebAuthn）与 TOTP 真实登录。
- 新增 `/auth/me` 用于获取当前用户角色与基础信息。
- 更新 Pinia 与路由守卫以使用服务端角色进行权限保护。

3) PDF 模板排版优化
- 使用表格/分页渲染记录摘要。
- 签名区域布局与图片位置固定，分页时自动换页。

## Verification
- Frontend: `pnpm test:unit`
- Backend: `cargo test`（若涉及后端改动）

## Rollback
- 使用 git 回退本次修改文件。

## Current Task: Features update from features.md
1) 竞赛名称导入与公开读取
- 新增竞赛库 Excel 导入接口与前端入口。
- 新增登录用户可读取竞赛库接口，学生端下拉选择竞赛名称。

2) 自定义字段与布局支持
- 新增表单字段查询接口（按类型）。
- 记录创建/查询/导出支持自定义字段值。
- 前端学生填报页面动态渲染自定义字段。

3) 批量导入志愿/竞赛记录
- 管理员可通过 Excel 批量导入志愿服务与竞赛获奖记录。
- 支持自定义字段列映射到配置字段。

## Verification
- Frontend: `pnpm test:unit`
- Backend: `cargo test`

## Rollback
- 使用 git 回退本次修改文件。

## Current Task: 全面测试覆盖 + CI（按 1→2→3 顺序）
阶段 1：后端单元测试扩展（优先完成）
- 为各模块内部函数补齐测试（auth/tls/records/students/attachments/exports/admin/forms 等）。
- 覆盖成功/失败/边界条件、权限控制、输入校验。
- 进度：已补齐 exports/admin/records/attachments/students 的纯函数与权限/校验测试（forms 仍缺少可测的纯函数）。

阶段 2：前端集成测试
- 路由权限、登录流程（Passkey/TOTP stub）、表单校验与错误提示、导入/导出交互。
- 维持已有视图单测，补齐关键交互与错误提示断言。
 - 进度：已补齐登录流程（Passkey/TOTP/恢复码）与路由守卫的集成测试。

阶段 3：CI 与后端集成测试强化
- 覆盖完整流程：管理员引导/登录、TOTP/Passkey、学生导入、记录提交、审核、导出、附件上传。
- SQLite 本地跑；CI 以 MySQL/PostgreSQL 运行同样的集成测试。
- GitHub Actions 矩阵运行后端/前端测试。
 - 进度：已补齐附件上传与签名上传的集成测试。

## Verification
- Frontend: `pnpm test:unit`
- Backend: `cargo test`
- CI: GitHub Actions（MySQL/PostgreSQL 服务）

## Goal
Implement the core backend + frontend skeleton for the VolunteerHours system based on functionlist.md, with secure auth (Passkey/TOTP), local file storage, and detailed API docs.

## Steps
1) Context & API design
- Confirm data model and core routes.
- Define auth flows (Passkey/TOTP/recovery code/device management).
- Draft OpenAPI schema.

2) Backend foundation
- Set up Axum app, configuration, error handling, logging.
- Add SeaORM, migrations, and entities for users/roles/students/volunteer/contest/review/attachments/devices/recovery codes.
- Implement RBAC middleware + request validation.

3) Auth & security
- Implement Passkey (WebAuthn) + TOTP verification.
- Enforce 2FA at login; add device management + recovery code flows.
- TLS support: default self-signed cert; allow import encrypted key.

4) Feature APIs
- Student import, query, and search.
- Volunteer/contest submissions with attachments.
- Review workflow (self/initial/final, signatures).
- Export PDF/Excel endpoints.

5) Frontend skeleton
- Vue3 routes & layouts for login/2FA, student portal, reviewer, admin.
- API client scaffolding.

6) Tests & docs
- Unit tests for auth/validation/permissions.
- API docs (OpenAPI + README usage).

## Verification
- Backend: cargo test, run server, basic health checks.
- Frontend: pnpm test:unit, manual route check.

## Rollback
- Revert changed files via git.

## Current Task: 邀请注册 + 学生密码策略 + 初始化 TOTP 强制
1) 初始化流程与二维码
- Setup 页面展示 TOTP 二维码与密钥。
- 引导流程强制完成 TOTP 后才允许进入登录页。
 - 进度：已完成（Setup 向导支持二维码与强制绑定）。

2) 用户体系与邀请注册
- users 表补充邮箱、密码哈希、密码策略标记、是否允许密码登录等字段。
- 新增邀请表：记录邀请 token、角色、邮箱、过期时间、使用状态。
- 管理员创建非学生用户必须填写邮箱并发送邀请邮件；受邀用户注册时必须绑定 TOTP 或 Passkey。
- 非学生不提供密码登录入口。
 - 进度：后端/前端主流程已实现，待补充接口文档与联调验证。

3) 学生密码与重置规则
- 学生默认密码为 st+学号（按密码策略生成与校验）。
- 学生允许密码登录，支持绑定邮箱后自助重置密码。
- 非学生仅允许管理员发起 TOTP/Passkey 重置邀请。
- 进度：后端/前端已实现（含管理员密码策略配置与重置页面）。

4) 一次性重置码（内网模式）
- 管理员可生成一次性重置码，用于学生密码或非学生 TOTP/Passkey 重置。
- 由配置项 `reset_delivery` 控制使用邮件或重置码。
 - 进度：后端/前端已实现（含重置码入口页面与管理端生成）。

## Verification
- Frontend: pnpm test:unit
- Backend: cargo test
- 手动验证：初始化向导 TOTP、邀请注册、学生密码登录/重置

## Rollback
- Revert changed files via git.
