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
