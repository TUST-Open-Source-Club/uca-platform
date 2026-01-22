# Labor Hours Platform 接口文档

本文档描述服务端已实现的接口与使用方式。

## 基础信息
- 基础地址：`https://<host>:<port>`（开发者模式或允许 HTTP 时可为 `http://<host>:<port>`）。
- 所有响应默认均为 JSON，除非特别说明。
- Cookie 为 HTTP-only，`SameSite=Strict`，默认 `Secure`；当 `ALLOW_HTTP=true` 时不设置 `Secure`。

## 错误格式
```json
{
  "code": "auth_error",
  "message": "认证失败：会话无效"
}
```

## 环境配置
- `BIND_ADDR`（默认 `0.0.0.0:8443`）
- `DATABASE_URL`（必填，支持 MySQL/PostgreSQL；开发者模式默认 SQLite）
- `RP_ID`（必填，WebAuthn RPID，如 `example.com`）
- `RP_ORIGIN`（必填，如 `https://example.com`）
- `TLS_CERT_PATH`（默认 `data/tls/cert.pem`）
- `TLS_KEY_PATH`（默认 `data/tls/key.enc`）
- `TLS_IMPORT_CERT_PEM`（可选，导入 PEM 证书）
- `TLS_IMPORT_KEY_PEM`（可选，导入 PEM 私钥）
- `TLS_KEY_ENC_KEY`（必填，Base64 解码后 32 字节）
- `AUTH_SECRET_KEY`（必填，Base64 解码后 32 字节）
- `UPLOAD_DIR`（默认 `data/uploads`）
- `SESSION_COOKIE_NAME`（默认 `vh_session`）
- `SESSION_TTL_SECONDS`（默认 `3600`）
- `BOOTSTRAP_TOKEN`（可选，引导创建管理员口令）
- `DEVELOPER_MODE`（可选，`true` 启用开发者模式，全部配置使用默认值）
- `ALLOW_HTTP`（可选，`true` 时允许 HTTP 直连；生产建议由反向代理终止 HTTPS）

开发者模式说明：
- 仅用于本地调试，会自动使用默认配置并生成自签名证书。
- 默认数据库为 `sqlite://data/dev.db?mode=rwc`，默认 RP 信息为 `localhost`/`http://localhost:8443`。
- 若启用 `ALLOW_HTTP=true`，服务以 HTTP 启动，HTTPS 交由反向代理处理。
- 开发者模式会忽略环境变量配置，强制使用默认值。
- `RESET_DELIVERY` 或配置文件 `reset_delivery` 可选值：`email` 或 `code`。

## 认证接口

### GET /health
健康检查接口。

响应：
```json
{ "status": "ok" }
```

### GET /auth/bootstrap/status
获取初始化状态（无需登录）。

响应：
```json
{ "ready": true, "needs_totp": false }
```

### POST /auth/bootstrap
创建初始管理员用户，仅在系统无用户时允许。若配置了 `BOOTSTRAP_TOKEN`，必须提供。成功后会写入会话 Cookie，用于在初始化阶段绑定 TOTP。

请求：
```json
{
  "token": "可选引导口令",
  "username": "admin",
  "display_name": "管理员"
}
```

响应：
```json
{ "user_id": "<uuid>" }
```

说明：
- 返回会话 Cookie（`Set-Cookie`），用于调用 `/auth/totp/enroll/start` 与 `/auth/totp/enroll/finish` 完成 TOTP 绑定。

### GET /auth/login/options
获取用户允许的登录方式（无需登录）。

请求：
```
/auth/login/options?username=20231234
```

响应：
```json
{ "methods": ["passkey", "totp", "password"] }
```

### GET /auth/password-policy
获取密码策略（用于前端提示，无需登录）。

响应：
```json
{
  "min_length": 8,
  "require_uppercase": false,
  "require_lowercase": false,
  "require_digit": true,
  "require_symbol": false
}
```

### POST /auth/reauth/password
使用当前密码进行二次验证，返回短效 reauth token（需要会话 Cookie）。

请求：
```json
{ "current_password": "当前密码" }
```

响应：
```json
{ "token": "<reauth_token>", "expires_in": 300 }
```

### POST /auth/reauth/totp
使用 TOTP 进行二次验证，返回短效 reauth token（需要会话 Cookie）。

请求：
```json
{ "code": "123456" }
```

响应：
```json
{ "token": "<reauth_token>", "expires_in": 300 }
```

### POST /auth/reauth/passkey/start
发起 Passkey 二次验证（需要会话 Cookie）。

响应：
```json
{
  "session_id": "<uuid>",
  "public_key": { "publicKey": { /* WebAuthn RequestChallengeResponse */ } }
}
```

### POST /auth/reauth/passkey/finish
完成 Passkey 二次验证，返回短效 reauth token。

请求：
```json
{
  "session_id": "<uuid>",
  "credential": { /* PublicKeyCredential */ }
}
```

响应：
```json
{ "token": "<reauth_token>", "expires_in": 300 }
```

### POST /auth/passkey/register/start
为已有用户发起 Passkey 注册（需要会话 Cookie，且用户名必须与当前会话一致）。

请求：
```json
{ "username": "20231234" }
```

响应：
```json
{
  "session_id": "<uuid>",
  "public_key": { "publicKey": { /* WebAuthn CreationChallengeResponse */ } }
}
```

### POST /auth/passkey/register/finish
完成 Passkey 注册。若用户已有任一凭据（密码/TOTP/Passkey），需携带二次验证头 `X-Reauth-Token`。

请求：
```json
{
  "session_id": "<uuid>",
  "credential": { /* RegisterPublicKeyCredential */ },
  "device_label": "我的设备（可选）"
}
```

响应：
```json
{ "passkey_id": "<uuid>" }
```

### POST /auth/passkey/login/start
发起 Passkey 登录。

请求：
```json
{ "username": "20231234" }
```

响应：
```json
{
  "session_id": "<uuid>",
  "public_key": { "publicKey": { /* WebAuthn RequestChallengeResponse */ } }
}
```

### POST /auth/passkey/login/finish
完成 Passkey 登录，成功后写入会话 Cookie。

请求：
```json
{
  "session_id": "<uuid>",
  "credential": { /* PublicKeyCredential */ }
}
```

响应：
```json
{ "user_id": "<uuid>" }
```

### POST /auth/password/login
学生密码登录（仅学生）。

请求：
```json
{ "username": "20231234", "password": "st20231234" }
```

响应：
```json
{ "user_id": "<uuid>" }
```

### GET /auth/me
获取当前会话用户信息。

响应：
```json
{
  "id": "<uuid>",
  "username": "20231234",
  "display_name": "张三",
  "role": "student"
}
```

### POST /auth/totp/enroll/start
为当前用户发起 TOTP 绑定（需要会话 Cookie）。

请求：
```json
{ "device_label": "验证器" }
```

响应：
```json
{
  "enrollment_id": "<uuid>",
  "otpauth_url": "otpauth://totp/..."
}
```

### POST /auth/totp/enroll/finish
完成 TOTP 绑定（需要会话 Cookie）。若用户已有任一凭据（密码/TOTP/Passkey），需携带二次验证头 `X-Reauth-Token`。

请求：
```json
{
  "enrollment_id": "<uuid>",
  "code": "123456"
}
```

响应：
```json
{ "status": "ok" }
```

### POST /auth/totp/verify
验证 TOTP 并创建会话。

请求：
```json
{
  "username": "20231234",
  "code": "123456"
}
```

响应：
```json
{ "user_id": "<uuid>" }
```

### POST /auth/recovery/verify
验证恢复码并创建会话。

请求：
```json
{
  "username": "20231234",
  "code": "<恢复码>"
}
```

响应：
```json
{ "user_id": "<uuid>" }
```

### POST /auth/email/bind
学生绑定邮箱（需要会话 Cookie）。

请求：
```json
{ "email": "student@example.com" }
```

响应：
```json
{ "status": "ok" }
```

### POST /auth/password/change
学生修改密码（需要会话 Cookie）。

请求：
```json
{ "current_password": "st20231234", "new_password": "NewPass123" }
```

响应：
```json
{ "status": "ok" }
```

### POST /auth/password/reset/request
学生发起密码重置邮件（无需登录，需先绑定邮箱）。当 `reset_delivery=code` 时该接口不可用。

请求：
```json
{ "username": "20231234" }
```

响应：
```json
{ "status": "ok" }
```

### POST /auth/password/reset/confirm
完成学生密码重置（无需登录，可来自邮件链接或一次性重置码）。

请求：
```json
{ "token": "<reset-token>", "new_password": "NewPass123" }
```

响应：
```json
{ "status": "ok" }
```

### GET /auth/invite/status
查询邀请状态（无需登录）。

请求：
```
/auth/invite/status?token=<invite-token>
```

响应：
```json
{
  "valid": true,
  "email": "user@example.com",
  "username": "teacher001",
  "display_name": "李老师",
  "role": "teacher",
  "expires_at": "2025-02-12T08:00:00Z"
}
```

### POST /auth/invite/accept
接受邀请并创建用户（无需登录，成功后写入会话 Cookie）。

请求：
```json
{ "token": "<invite-token>" }
```

响应：
```json
{ "user_id": "<uuid>", "username": "teacher001", "role": "teacher" }
```

### GET /auth/reset/status
查询认证重置令牌状态（无需登录）。

请求：
```
/auth/reset/status?token=<reset-token>
```

响应：
```json
{ "valid": true, "purpose": "totp" }
```

### POST /auth/reset/consume
消费认证重置令牌并清理认证数据（无需登录，成功后写入会话 Cookie）。

请求：
```json
{ "token": "<reset-token>" }
```

响应：
```json
{ "user_id": "<uuid>", "purpose": "totp" }
```

（已移除用户自助恢复码生成接口）

### GET /auth/devices
列出当前用户已绑定设备（需要会话 Cookie）。

响应：
```json
[
  {
    "id": "<uuid>",
    "user_id": "<uuid>",
    "device_type": "passkey",
    "label": "Passkey-...",
    "credential_id": "<base64url>",
    "created_at": "...",
    "last_used_at": "..."
  }
]
```

### DELETE /auth/devices/{device_id}
移除当前用户设备。若用户已有任一凭据（密码/TOTP/Passkey），需携带二次验证头 `X-Reauth-Token`。

响应：
```json
{ "status": "ok" }
```

### GET /profile/signature
获取当前用户签名图片状态（审核/管理员/教师）。

响应：
```json
{ "uploaded": true, "signature_path": "data/uploads/signatures/users/<user_id>/signature_20250101.png" }
```

### POST /profile/signature
上传当前用户签名图片（审核/管理员/教师，multipart 字段 `file`）。

响应：
```json
{ "uploaded": true, "signature_path": "data/uploads/signatures/users/<user_id>/signature_20250101.png" }
```

## 学生接口

### POST /students
创建学生（仅管理员，需会话 Cookie）。

请求：
```json
{
  "student_no": "2023001",
  "name": "张三",
  "gender": "男",
  "department": "信息学院",
  "major": "软件工程",
  "class_name": "软工1班",
  "phone": "13800000000"
}
```

### PUT /students/{student_no}
更新学生信息（仅管理员，需会话 Cookie）。

请求：
```json
{
  "name": "张三",
  "gender": "男",
  "department": "信息学院",
  "major": "软件工程",
  "class_name": "软工1班",
  "phone": "13800000000"
}
```

响应：同创建学生响应结构。

响应：
```json
{
  "id": "<uuid>",
  "student_no": "2023001",
  "name": "张三",
  "gender": "男",
  "department": "信息学院",
  "major": "软件工程",
  "class_name": "软工1班",
  "phone": "13800000000"
}
```

### POST /students/query
按条件查询学生（管理员/教师/审核人员），过滤条件通过 JSON 请求体传入。

请求：
```json
{
  "department": "信息学院",
  "major": "软件工程",
  "class_name": "软工1班",
  "keyword": "2023"
}
```

响应：
```json
[{
  "id": "<uuid>",
  "student_no": "2023001",
  "name": "张三",
  "gender": "男",
  "department": "信息学院",
  "major": "软件工程",
  "class_name": "软工1班",
  "phone": "13800000000"
}]
```

### POST /students/import
从 Excel 导入学生（仅管理员），multipart 字段 `file`，可选 `field_map` 指定字段映射。

请求： `multipart/form-data`
- `file`：腾讯文档导出的 `.xlsx` 文件
- `field_map`（可选）：JSON 字符串，指定字段到列的映射
- `allow_login`（可选）：`true/false`，是否允许学生登录（默认 `false`）

响应：
```json
{ "inserted": 120, "updated": 5 }
```

`field_map` 示例（列可为表头/列字母/列序号）：
```json
{
  "student_no": "学号",
  "name": "B",
  "major": "专业"
}
```

`allow_login` 示例：
```
allow_login=true
```

标准表头（学生导入）：
```
学号 | 姓名 | 性别 | 院系 | 专业 | 班级 | 手机号
```
示例（第一行表头，后续为数据行）：
```
学号,姓名,性别,院系,专业,班级,手机号
2023001,张三,男,信息学院,软件工程,软工1班,13800000000
```

## 记录接口

### POST /records/contest
提交竞赛获奖记录（学生）。

请求：
```json
{
  "contest_year": 2024,
  "contest_category": "A",
  "contest_name": "全国大学生数学建模竞赛",
  "contest_level": "国家级",
  "contest_role": "负责人",
  "award_level": "省赛一等奖",
  "award_date": "2024-10-20",
  "self_hours": 8,
  "custom_fields": {
    "sponsor": "数学学院"
  }
}
```

响应：
```json
{
  "id": "<uuid>",
  "student_id": "<uuid>",
  "contest_year": 2024,
  "contest_category": "A",
  "contest_name": "全国大学生数学建模竞赛",
  "contest_level": "国家级",
  "contest_role": "负责人",
  "award_level": "省赛一等奖",
  "award_date": "2024-10-20T00:00:00+00:00",
  "self_hours": 8,
  "first_review_hours": null,
  "final_review_hours": null,
  "status": "submitted",
  "rejection_reason": null,
  "match_status": "matched",
  "recommended_hours": 6,
  "custom_fields": [
    { "field_key": "sponsor", "label": "主办方", "value": "数学学院" }
  ]
}
```

### POST /records/contest/query
查询竞赛记录（学生/审核角色）。

请求：
```json
{ "status": "submitted" }
```

### POST /records/contest/{record_id}/review
审核竞赛记录（初审：审核人员/管理员；复审：教师/管理员）。

请求：
```json
{
  "stage": "final",
  "hours": 6,
  "status": "approved",
  "rejection_reason": null
}
```

## 附件与签名

### POST /attachments/contest/{record_id}
上传竞赛附件（学生本人，multipart `file`）。

响应：
```json
{ "id": "<uuid>", "stored_name": "..." }
```

### POST /signatures/{record_type}/{record_id}/{stage}
上传审核签名（stage: first/final）。

响应：
```json
{ "id": "<uuid>", "signature_path": "..." }
```

## 导出

### POST /export/summary/excel
导出学院/专业/班级汇总表。

请求：
```json
{ "department": "信息学院", "major": "软件工程", "class_name": "软工1班" }
```

汇总导出字段支持自定义（通过 `form_fields` 的 `form_type=summary` 配置），内置字段 key：
```
student_no | name | gender | department | major | class_name | phone | self_hours | approved_hours | reason
```

### POST /export/student/{student_no}/excel
导出个人学时专项表。

个人导出字段支持自定义（通过 `form_fields` 的 `form_type=student_export` 配置），内置字段 key：
```
student_no | name | gender | department | major | class_name | phone | self_hours | approved_hours | reason
```

### POST /export/labor-hours/summary/excel
导出劳动教育学时汇总表（Excel）。

请求：
```json
{ "department": "信息学院", "major": "软件工程", "class_name": "软工1班" }
```

表头支持自定义（通过 `form_fields` 的 `form_type=labor_hours_excel` 配置），内置字段 key：
```
index | major | class_name | student_no | name | planned_hours | module_hours | reason
```

### POST /export/record/{record_type}/{record_id}/pdf
导出单条记录 PDF。
说明：`record_type` 仅支持 `contest`。

### POST /export/labor-hours/{student_no}/pdf
导出劳动教育学时认定表（每学生一份 PDF）。该 PDF 的字段与布局由导出模板配置决定。

## 管理接口

### GET /forms/{form_type}/fields
获取指定表单类型字段（需登录）。

响应：
```json
[
  {
    "id": "<uuid>",
    "form_type": "contest",
    "field_key": "location",
    "label": "地点",
    "field_type": "text",
    "required": true,
    "order_index": 1
  }
]
```

### GET /competitions
获取竞赛名称库（无需登录，只读）。

响应：
```json
[
  { "id": "<uuid>", "year": 2024, "category": "A", "name": "全国大学生数学建模竞赛" }
]
```

### POST /admin/users
管理员创建用户或发送邀请（需会话 Cookie）。

说明：
- 角色为 `student` 时直接创建用户并设置默认密码 `st+学号`。
- 其他角色必须提供邮箱，系统发送邀请邮件。

请求：
```json
{
  "username": "teacher001",
  "display_name": "李老师",
  "role": "teacher",
  "email": "teacher@example.com"
}
```

响应：
```json
{ "user_id": null, "invite_sent": true }
```

### GET /admin/password-policy
获取密码策略（需会话 Cookie）。

响应：
```json
{
  "min_length": 8,
  "require_uppercase": false,
  "require_lowercase": false,
  "require_digit": true,
  "require_symbol": false
}
```

### POST /admin/password-policy
更新密码策略（需会话 Cookie）。

请求：
```json
{
  "min_length": 8,
  "require_uppercase": false,
  "require_lowercase": false,
  "require_digit": true,
  "require_symbol": false
}
```

响应：
```json
{
  "min_length": 8,
  "require_uppercase": false,
  "require_lowercase": false,
  "require_digit": true,
  "require_symbol": false
}
```

### POST /admin/users/reset/totp
发送 TOTP 重置链接（仅非学生，需会话 Cookie）。

请求：
```json
{ "username": "teacher001" }
```

响应：
```json
{ "status": "ok" }
```

### POST /admin/users/reset/passkey
发送 Passkey 重置链接（仅非学生，需会话 Cookie）。

请求：
```json
{ "username": "teacher001" }
```

响应：
```json
{ "status": "ok" }
```

### POST /admin/users/reset/code
生成一次性重置码（仅内网模式，需会话 Cookie）。

说明：
- `purpose=password` 仅学生可用。
- `purpose=totp/passkey` 仅非学生可用。
- 当 `reset_delivery=email` 时该接口不可用。

请求：
```json
{ "username": "teacher001", "purpose": "totp" }
```

响应：
```json
{ "code": "ABCD1234", "expires_in_minutes": 1440 }
```

### GET /admin/competitions
获取竞赛名称库（管理员）。

### POST /admin/competitions
新增竞赛名称（管理员）。

### PUT /admin/competitions/{competition_id}
更新竞赛名称（管理员）。

### DELETE /admin/competitions/{competition_id}
删除竞赛名称（管理员）。

请求：
```json
{ "name": "全国大学生数学建模竞赛" }
```

### POST /admin/competitions/import
从 Excel 导入竞赛名称（管理员，multipart 字段 `file`）。

可选字段：
- `default_year`：年份列缺失时的默认年份。
- `sheet_plan`：JSON 数组，指定导入的工作表与年份，例如：
```json
[
  {
    "name": "Sheet1",
    "year": 2024,
    "name_column": "竞赛名称",
    "category_column": "竞赛类型",
    "category_suffix": "class"
  },
  {
    "name": "Sheet2",
    "year": 2023,
    "name_column": "B",
    "category_column": "C",
    "category_suffix": "class_contest"
  }
]
```

响应：
```json
{ "inserted": 10, "skipped": 2 }
```

说明：
- `sheet_plan` 用于选择工作表并设置年份/列映射，可用表头名或列字母/序号。
- 未提供 `sheet_plan` 时默认导入第一个工作表。
- 若表格没有年份列且未设置 `year`，将返回提示错误。
- `category_suffix` 可选值：`class`（去掉“类”后缀）、`class_contest`（去掉“类竞赛”后缀）。

### GET /admin/form-fields
获取表单字段配置（管理员）。

### POST /admin/form-fields
新增表单字段（管理员）。

请求：
```json
{
  "form_type": "contest",
  "field_key": "location",
  "label": "地点",
  "field_type": "text",
  "required": true,
  "order_index": 1
}
```

表单类型（form_type）建议值：
```
contest | summary | student_export
```

### GET /admin/export-templates/{template_key}
获取导出模板（管理员）。

响应：
```json
{ "template_key": "labor_hours", "name": "labor-hours.xlsx", "issues": [] }
```

### POST /admin/export-templates/{template_key}/upload
上传导出模板（管理员，Excel 格式）。

说明：
- 该模板用于导出 PDF，模板合法性校验后保存。
- 响应会返回校验问题列表。
- 占位符规则详见 README.md。
- 请求为 multipart/form-data，包含 `file` 字段。

### GET /admin/labor-hour-rules
获取劳动学时规则（管理员）。

### POST /admin/labor-hour-rules
更新劳动学时规则（管理员）。

### GET /admin/deleted/students
获取已删除学生列表（管理员）。

### GET /admin/deleted/records/contest
获取已删除竞赛记录（管理员）。

### DELETE /admin/students/{student_no}
软删除学生（管理员）。

响应：
```json
{ "deleted": true }
```

说明：仅设置 `is_deleted=1`，已审核记录不会被删除。

### DELETE /admin/purge/students/{student_no}
彻底删除学生（管理员，仅允许删除已软删除的学生）。

响应：
```json
{ "deleted": true }
```

### DELETE /admin/records/contest/{record_id}
软删除未审核的竞赛记录（管理员）。

响应：
```json
{ "deleted": true }
```

说明：仅允许删除 `status=submitted` 的记录。

### DELETE /admin/purge/records/contest/{record_id}
彻底删除竞赛记录（管理员，仅允许删除已软删除的记录）。

响应：
```json
{ "deleted": true }
```

### POST /admin/records/contest/import
批量导入竞赛获奖记录（管理员，multipart 字段 `file`，可选 `field_map`）。

响应：
```json
{ "inserted": 10, "skipped": 1 }
```

`field_map` 示例（列可为表头/列字母/列序号）：
```json
{
  "student_no": "学号",
  "contest_name": "竞赛名称",
  "contest_level": "C",
  "self_hours": "F"
}
```

标准表头（竞赛获奖导入，默认必填，可在导入模板中调整）：
```
学号 | 竞赛名称 | 竞赛级别 | 角色 | 获奖等级 | 自评学时
```
常用可选表头：
```
竞赛年份 | 竞赛类型 | 获奖时间 | 初审学时 | 复审学时 | 审核状态 | 不通过原因
```
自定义字段列：
```
列名可为字段 label 或 field_key（如 “主办方” 或 “sponsor”）
```
示例：
```
学号,竞赛名称,竞赛级别,角色,获奖等级,自评学时,竞赛年份,竞赛类型,获奖时间,复审学时,审核状态,主办方
2023001,全国大学生数学建模竞赛,国家级,负责人,省赛一等奖,8,2024,A,2024-10-20,6,已复审,数学学院
```
