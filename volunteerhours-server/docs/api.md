# VolunteerHours 接口文档

本文档描述服务端已实现的接口与使用方式。

## 基础信息
- 基础地址：`https://<host>:<port>`（开发者模式或允许 HTTP 时可为 `http://<host>:<port>`）。
- 所有响应默认均为 JSON，除非特别说明。
- Cookie 为 HTTP-only，`SameSite=Strict`，默认 `Secure`；当 `ALLOW_HTTP=true` 时不设置 `Secure`。

## 错误格式
```json
{
  "code": "auth_error",
  "message": "auth error: invalid session"
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

## 认证接口

### GET /health
健康检查接口。

响应：
```json
{ "status": "ok" }
```

### POST /auth/bootstrap
创建初始管理员用户，仅在系统无用户时允许。若配置了 `BOOTSTRAP_TOKEN`，必须提供。

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

### POST /auth/passkey/register/start
为已有用户发起 Passkey 注册。

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
完成 Passkey 注册。

请求：
```json
{
  "session_id": "<uuid>",
  "credential": { /* RegisterPublicKeyCredential */ },
  "device_label": "我的设备"
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
完成 TOTP 绑定（需要会话 Cookie）。

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

### POST /auth/recovery/regenerate
重新生成恢复码（需要会话 Cookie），旧恢复码将失效。

响应：
```json
{ "codes": ["恢复码1", "恢复码2", "..."] }
```

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
移除当前用户设备。

响应：
```json
{ "status": "ok" }
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
从 Excel 导入学生（仅管理员），multipart 字段 `file`。

请求： `multipart/form-data`
- `file`：腾讯文档导出的 `.xlsx` 文件

响应：
```json
{ "inserted": 120, "updated": 5 }
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

### POST /records/volunteer
提交志愿服务记录（学生）。

请求：
```json
{
  "title": "社区服务",
  "description": "社区清洁",
  "self_hours": 4,
  "custom_fields": {
    "location": "校内操场"
  }
}
```

响应：
```json
{
  "id": "<uuid>",
  "student_id": "<uuid>",
  "title": "社区服务",
  "description": "社区清洁",
  "self_hours": 4,
  "first_review_hours": null,
  "final_review_hours": null,
  "status": "submitted",
  "rejection_reason": null,
  "custom_fields": [
    { "field_key": "location", "label": "地点", "value": "校内操场" }
  ]
}
```

### POST /records/contest
提交竞赛获奖记录（学生）。

请求：
```json
{
  "contest_name": "全国大学生数学建模竞赛",
  "award_level": "省赛一等奖",
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
  "contest_name": "全国大学生数学建模竞赛",
  "award_level": "省赛一等奖",
  "self_hours": 8,
  "first_review_hours": null,
  "final_review_hours": null,
  "status": "submitted",
  "rejection_reason": null,
  "match_status": "matched",
  "custom_fields": [
    { "field_key": "sponsor", "label": "主办方", "value": "数学学院" }
  ]
}
```

### POST /records/volunteer/query
查询志愿服务记录（学生/审核角色）。

请求：
```json
{ "status": "submitted" }
```

### POST /records/contest/query
查询竞赛记录（学生/审核角色）。

请求：
```json
{ "status": "submitted" }
```

### POST /records/volunteer/{record_id}/review
审核志愿服务记录（初审：审核人员/管理员；复审：教师/管理员）。

请求：
```json
{
  "stage": "first",
  "hours": 3,
  "status": "approved",
  "rejection_reason": null
}
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

### POST /attachments/volunteer/{record_id}
上传志愿服务附件（学生本人，multipart `file`）。

响应：
```json
{ "id": "<uuid>", "stored_name": "..." }
```

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

### POST /export/record/{record_type}/{record_id}/pdf
导出单条记录 PDF。

## 管理接口

### GET /forms/{form_type}/fields
获取指定表单类型字段（需登录）。

响应：
```json
[
  {
    "id": "<uuid>",
    "form_type": "volunteer",
    "field_key": "location",
    "label": "地点",
    "field_type": "text",
    "required": true,
    "order_index": 1
  }
]
```

### GET /competitions
获取竞赛名称库（需登录）。

响应：
```json
[
  { "id": "<uuid>", "name": "全国大学生数学建模竞赛" }
]
```

### GET /admin/competitions
获取竞赛名称库（管理员）。

### POST /admin/competitions
新增竞赛名称（管理员）。

请求：
```json
{ "name": "全国大学生数学建模竞赛" }
```

### POST /admin/competitions/import
从 Excel 导入竞赛名称（管理员，multipart 字段 `file`）。

响应：
```json
{ "inserted": 10, "skipped": 2 }
```

标准表头（竞赛库导入）：
```
竞赛名称
```
示例：
```
竞赛名称
全国大学生数学建模竞赛
```

### GET /admin/form-fields
获取表单字段配置（管理员）。

### POST /admin/form-fields
新增表单字段（管理员）。

请求：
```json
{
  "form_type": "volunteer",
  "field_key": "location",
  "label": "地点",
  "field_type": "text",
  "required": true,
  "order_index": 1
}
```

表单类型（form_type）建议值：
```
volunteer | contest | summary | student_export
```

### POST /admin/records/volunteer/import
批量导入志愿服务记录（管理员，multipart 字段 `file`）。

响应：
```json
{ "inserted": 12, "skipped": 3 }
```

标准表头（志愿服务导入，必填）：
```
学号 | 标题 | 描述 | 自评学时
```
可选表头：
```
初审学时 | 复审学时 | 审核状态 | 不通过原因
```
自定义字段列：
```
列名可为字段 label 或 field_key（如 “地点” 或 “location”）
```
示例：
```
学号,标题,描述,自评学时,初审学时,审核状态,地点
2023001,社区服务,社区清洁,4,3,已初审,校内操场
```

### POST /admin/records/contest/import
批量导入竞赛获奖记录（管理员，multipart 字段 `file`）。

响应：
```json
{ "inserted": 10, "skipped": 1 }
```

标准表头（竞赛获奖导入，必填）：
```
学号 | 竞赛名称 | 获奖等级 | 自评学时
```
可选表头：
```
初审学时 | 复审学时 | 审核状态 | 不通过原因
```
自定义字段列：
```
列名可为字段 label 或 field_key（如 “主办方” 或 “sponsor”）
```
示例：
```
学号,竞赛名称,获奖等级,自评学时,复审学时,审核状态,主办方
2023001,全国大学生数学建模竞赛,省赛一等奖,8,6,已复审,数学学院
```
