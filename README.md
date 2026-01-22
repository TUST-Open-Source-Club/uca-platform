# 劳动教育学时认定系统（Labor Hours Platform）

本仓库包含前端（`uca-platform-ui`）与后端（`uca-platform-server`）。

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
