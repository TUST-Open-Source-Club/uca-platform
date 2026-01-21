# VolunteerHours API

This document describes the current authentication APIs implemented in the server.

## Base
- Base URL: `https://<host>:<port>`
- All responses are JSON unless noted otherwise.
- Cookies are HTTP-only, `SameSite=Strict`, and `Secure`.

## Error format
```json
{
  "code": "auth_error",
  "message": "auth error: invalid session"
}
```

## Environment configuration
- `BIND_ADDR` (default `0.0.0.0:8443`)
- `DATABASE_URL` (required, MySQL or PostgreSQL)
- `RP_ID` (required, WebAuthn RPID, e.g. `example.com`)
- `RP_ORIGIN` (required, e.g. `https://example.com`)
- `TLS_CERT_PATH` (default `data/tls/cert.pem`)
- `TLS_KEY_PATH` (default `data/tls/key.enc`)
- `TLS_IMPORT_CERT_PEM` (optional PEM import)
- `TLS_IMPORT_KEY_PEM` (optional PEM import)
- `TLS_KEY_ENC_KEY` (required base64, 32 bytes after decode)
- `AUTH_SECRET_KEY` (required base64, 32 bytes after decode)
- `UPLOAD_DIR` (default `data/uploads`)
- `SESSION_COOKIE_NAME` (default `vh_session`)
- `SESSION_TTL_SECONDS` (default `3600`)
- `BOOTSTRAP_TOKEN` (optional bootstrap secret)

## Auth endpoints

### GET /health
Simple health check.

Response:
```json
{ "status": "ok" }
```

### POST /auth/bootstrap
Create the initial admin user. Only allowed when no users exist. If `BOOTSTRAP_TOKEN` is set, it must be provided.

Request:
```json
{
  "token": "<optional bootstrap token>",
  "username": "admin",
  "display_name": "Admin"
}
```

Response:
```json
{ "user_id": "<uuid>" }
```

### POST /auth/passkey/register/start
Start passkey registration for an existing user.

Request:
```json
{ "username": "20231234" }
```

Response:
```json
{
  "session_id": "<uuid>",
  "public_key": { "publicKey": { /* WebAuthn CreationChallengeResponse */ } }
}
```

### POST /auth/passkey/register/finish
Finish passkey registration.

Request:
```json
{
  "session_id": "<uuid>",
  "credential": { /* RegisterPublicKeyCredential */ },
  "device_label": "My Laptop"
}
```

Response:
```json
{ "passkey_id": "<uuid>" }
```

### POST /auth/passkey/login/start
Start passkey authentication.

Request:
```json
{ "username": "20231234" }
```

Response:
```json
{
  "session_id": "<uuid>",
  "public_key": { "publicKey": { /* WebAuthn RequestChallengeResponse */ } }
}
```

### POST /auth/passkey/login/finish
Finish passkey authentication. On success, sets a session cookie.

Request:
```json
{
  "session_id": "<uuid>",
  "credential": { /* PublicKeyCredential */ }
}
```

Response:
```json
{ "user_id": "<uuid>" }
```

### GET /auth/me
Return current session user profile.

Response:
```json
{
  "id": "<uuid>",
  "username": "20231234",
  "display_name": "张三",
  "role": "student"
}
```

### POST /auth/totp/enroll/start
Start TOTP enrollment for the current user (requires session cookie).

Request:
```json
{ "device_label": "Authenticator" }
```

Response:
```json
{
  "enrollment_id": "<uuid>",
  "otpauth_url": "otpauth://totp/..."
}
```

### POST /auth/totp/enroll/finish
Finish TOTP enrollment (requires session cookie).

Request:
```json
{
  "enrollment_id": "<uuid>",
  "code": "123456"
}
```

Response:
```json
{ "status": "ok" }
```

### POST /auth/totp/verify
Verify TOTP and create a session.

Request:
```json
{
  "username": "20231234",
  "code": "123456"
}
```

Response:
```json
{ "user_id": "<uuid>" }
```

### POST /auth/recovery/verify
Verify a recovery code and create a session.

Request:
```json
{
  "username": "20231234",
  "code": "<recovery code>"
}
```

Response:
```json
{ "user_id": "<uuid>" }
```

### POST /auth/recovery/regenerate
Regenerate recovery codes for current user (requires session cookie). Old codes are invalidated.

Response:
```json
{ "codes": ["code1", "code2", "..."] }
```

### GET /auth/devices
List registered devices for current user (requires session cookie).

Response:
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
Remove a device for the current user.

Response:
```json
{ "status": "ok" }
```

## Student endpoints

### POST /students
Create a student (admin only, requires session cookie).

Request:
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

Response:
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
List students with filters (admin/teacher/reviewer). Use JSON body for filters.

Request:
```json
{
  "department": "信息学院",
  "major": "软件工程",
  "class_name": "软工1班",
  "keyword": "2023"
}
```

Response:
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
Import students from Excel (admin only). Multipart form field `file`.

Request: `multipart/form-data`
- `file`: `.xlsx` exported from Tencent Docs

Response:
```json
{ "inserted": 120, "updated": 5 }
```

## 记录接口

### POST /records/volunteer
提交志愿服务记录（学生）。

Request:
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

Response:
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

Request:
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

Response:
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

Request:
```json
{ "status": "submitted" }
```

### POST /records/contest/query
查询竞赛记录（学生/审核角色）。

Request:
```json
{ "status": "submitted" }
```

### POST /records/volunteer/{record_id}/review
审核志愿服务记录（初审 reviewer/admin，复审 teacher/admin）。

Request:
```json
{
  "stage": "first",
  "hours": 3,
  "status": "approved",
  "rejection_reason": null
}
```

### POST /records/contest/{record_id}/review
审核竞赛记录（初审 reviewer/admin，复审 teacher/admin）。

Request:
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

Response:
```json
{ "id": "<uuid>", "stored_name": "..." }
```

### POST /attachments/contest/{record_id}
上传竞赛附件（学生本人，multipart `file`）。

Response:
```json
{ "id": "<uuid>", "stored_name": "..." }
```

### POST /signatures/{record_type}/{record_id}/{stage}
上传审核签名（stage: first/final）。

Response:
```json
{ "id": "<uuid>", "signature_path": "..." }
```

## 导出

### POST /export/summary/excel
导出学院/专业/班级汇总表。

Request:
```json
{ "department": "信息学院", "major": "软件工程", "class_name": "软工1班" }
```

### POST /export/student/{student_no}/excel
导出个人学时专项表。

### POST /export/record/{record_type}/{record_id}/pdf
导出单条记录 PDF。

## 管理接口

### GET /forms/{form_type}/fields
获取指定表单类型字段（需登录）。

Response:
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

### GET /admin/competitions
获取竞赛名称库（管理员）。

### POST /admin/competitions
新增竞赛名称（管理员）。

Request:
```json
{ "name": "全国大学生数学建模竞赛" }
```

### POST /admin/competitions/import
从 Excel 导入竞赛名称（管理员，multipart 字段 `file`）。

Response:
```json
{ "inserted": 10, "skipped": 2 }
```

### GET /admin/form-fields
获取表单字段配置（管理员）。

### POST /admin/form-fields
新增表单字段（管理员）。

Request:
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

### POST /admin/records/volunteer/import
批量导入志愿服务记录（管理员，multipart 字段 `file`）。

Response:
```json
{ "inserted": 12, "skipped": 3 }
```

### POST /admin/records/contest/import
批量导入竞赛获奖记录（管理员，multipart 字段 `file`）。

Response:
```json
{ "inserted": 10, "skipped": 1 }
```
