# 劳动教育学时认定系统（Labor Hours Platform）

本仓库包含前端（`uca-platform-ui`）与后端（`uca-platform-server`）。

## 部署教程（Docker 一键部署）

### 1. 准备镜像

项目已提供 GHCR 镜像：
- `ghcr.io/mike-solar/uca-platform-server:latest`
- `ghcr.io/mike-solar/uca-platform-ui:latest`

如需自行构建：
```bash
docker compose build
```

### 2. 启动服务

```bash
docker compose up -d
```

默认对外端口：`http://localhost:8080`（由 Nginx 反代到前后端）。

### 3. 首次初始化

1. 打开 `http://localhost:8080`，进入初始化向导。
2. 初始化管理员需绑定 TOTP（系统要求首次初始化完成二次认证配置）。
3. 按界面提示完成后即可进入系统。

### 4. 数据库与存储

默认使用 PostgreSQL（容器 `uca-postgres`）。
数据卷：
- `postgres_data`：数据库数据
- `server_data`：后端上传文件/签名/证书等

### 5. 关键环境变量（务必修改）

`docker-compose.yml` 内含默认演示配置，请在生产环境中修改以下变量：
- `AUTH_SECRET_KEY`：Base64 的 32 字节密钥
- `TLS_KEY_ENC_KEY`：Base64 的 32 字节密钥
- `RP_ID` / `RP_ORIGIN` / `BASE_URL`：与部署域名一致
- `DATABASE_URL`：生产数据库连接串

### 6. HTTPS

默认允许 HTTP（`ALLOW_HTTP=true`），便于放在反向代理后。
如需后端直接启用 HTTPS，关闭 `ALLOW_HTTP` 并配置 TLS 证书路径：
- `TLS_CERT_PATH`
- `TLS_KEY_PATH`

> 生产环境推荐让 Nginx/反向代理终止 HTTPS。

## PDF 导出模板（Excel 占位符）

劳动教育学时认定表使用 Excel 模板导出，模板由管理员上传，后端替换占位符后通过 LibreOffice 转换为 PDF。

### 占位符语法

1) 单元格占位符（单值）：
- 形式：`{{字段}}`
- 可作为整格内容，也可以嵌入到文本中（例如：`姓名：{{name}}`）

2) 列表占位符（按列向下替换）：
- 形式：`{{list:字段}}`
- 该单元格所在列从当前行开始向下替换为当前学生的竞赛列表。
- 若空单元格不够，会自动增加新行。

3) 终止符：
- 形式：`{{/list}}`
- 仅当该列上方出现了 `{{list:...}}` 时生效。
- 终止符所在单元格会被替换为空，并且该列后续停止替换（即使列表未用完）。

### 可用字段

单值字段（用于 `{{字段}}`）：
- `student_no` 学号
- `name` 姓名
- `gender` 性别
- `department` 院系
- `major` 专业
- `class_name` 班级
- `phone` 手机号
- `total_self_hours` 自评学时合计
- `total_approved_hours` 审核通过学时合计
- `total_reason` 不通过原因汇总
- `first_signature_path` 初审教师签名路径（文本）
- `final_signature_path` 复审教师签名路径（文本）
- `first_signature_image` 初审电子签名图片（替换为图片）
- `final_signature_image` 复审电子签名图片（替换为图片）

列表字段（用于 `{{list:字段}}`）：
- `seq` 序号（从 1 递增）
- `contest_year` 竞赛年份
- `contest_category` 竞赛类别（A/B）
- `contest_name` 竞赛名称
- `contest_level` 竞赛级别（国家级/省级/校级）
- `contest_role` 角色（负责人/成员）
- `award_level` 获奖等级
- `award_date` 获奖时间
- `self_hours` 自评学时
- `first_review_hours` 初审学时
- `final_review_hours` 复审学时
- `approved_hours` 审核学时（等同于复审学时）
- `recommended_hours` 推荐学时
- `status` 审核状态
- `rejection_reason` 不通过原因
- `custom.<字段Key>` 竞赛自定义字段（例如：`custom.sponsor`）

### 个人中心签名

审核人员/管理员可在“个人中心”上传签名图片，用于导出 PDF 中的 `first_signature_image`/`final_signature_image` 占位符。

### LibreOffice 依赖

导出 PDF 依赖 LibreOffice。
- 默认使用 `soffice`。
- 可通过环境变量 `LIBREOFFICE_PATH` 或配置文件指定可执行路径。

## 竞赛库导入列映射

导入竞赛库时可在弹窗中配置：
- 工作表名称
- 年份（若 Excel 无年份列需填写）
- 竞赛名称列
- 竞赛类别列（可指定“类”或“类竞赛”后缀）

列可填写：
- 表头名称（例如：`竞赛名称`）
- 列字母（例如：`A`）
- 列序号（例如：`1`）
