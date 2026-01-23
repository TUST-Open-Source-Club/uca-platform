1) axum 里最重要：统一的 AuthN / AuthZ 机制（别在 handler 里手写 if）
建议结构

AuthN（认证）：从请求里解析用户身份 → 放进 Request extensions

AuthZ（授权）：用“策略函数/宏/守卫”做对象级权限校验（BOLA/IDOR 的核心）

middleware 示例（把 user 放进 extensions）：

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct AuthedUser {
    pub sub: String,      // user id
    pub roles: Vec<String>,
}

#[derive(Deserialize)]
struct Claims {
    sub: String,
    roles: Vec<String>,
    exp: usize,
}

pub async fn auth_mw<B>(mut req: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let auth = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let token = auth.strip_prefix("Bearer ").ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?
    .claims;

    req.extensions_mut().insert(AuthedUser {
        sub: claims.sub,
        roles: claims.roles,
    });

    Ok(next.run(req).await)
}


handler 里取用户：

use axum::Extension;

async fn me(Extension(u): Extension<AuthedUser>) -> String {
    format!("hello {}", u.sub)
}

2) 对象级权限（BOLA/IDOR）要用“可复用的 guard”，别每个接口写一遍

最常见事故：/users/:id、/docs/:id 这种只验证“已登录”，没验证“这个资源是否属于我/我可管理”。

SeaORM 对象级校验写法（先查归属，再执行）：

use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};

async fn ensure_owner(
    db: &sea_orm::DatabaseConnection,
    user_id: &str,
    doc_id: i64,
) -> Result<(), axum::http::StatusCode> {
    let owned = entity::doc::Entity::find_by_id(doc_id)
        .filter(entity::doc::Column::OwnerId.eq(user_id))
        .one(db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();

    if owned { Ok(()) } else { Err(axum::http::StatusCode::FORBIDDEN) }
}


关键点：“更新/删除/导出”之前必须先做对象级校验，而且批量接口要“逐条校验”。

3) 输入校验：别只靠 Rust 类型；要限制长度/枚举/范围/嵌套深度

axum/serde 能保证类型，但保证不了：

字符串超长（DoS）

枚举/范围错误（逻辑漏洞）

JSON 深层嵌套（DoS）

推荐：validator + 手动限制

use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
struct UpdateProfile {
    #[validate(length(min = 1, max = 50))]
    display_name: String,

    #[validate(phone)] // 或自己写 regex
    phone: Option<String>,
}


请求体大小限制（强烈建议）：

use axum::{Router};
use tower_http::limit::RequestBodyLimitLayer;

let app = Router::new()
    // ...
    .layer(RequestBodyLimitLayer::new(1024 * 1024)); // 1MB

4) SQL 注入：SeaORM 默认安全，但注意“动态 SQL / 原生查询”坑

QueryFilter / ActiveModel 这些是参数化的 ✅

危险点在：

Statement::from_string + 拼接字符串

把用户输入当列名/排序字段/表名

安全做法：排序字段用白名单映射

enum SortKey { CreatedAt, Name }

fn parse_sort(s: &str) -> SortKey {
    match s {
        "created_at" => SortKey::CreatedAt,
        "name" => SortKey::Name,
        _ => SortKey::CreatedAt,
    }
}

5) 错误处理：对外“模糊”，对内“可观测”

别把 DB 错误、堆栈、SQL 细节直接返回给用户。

建议统一 AppError：

use axum::{response::IntoResponse, http::StatusCode, Json};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    Unauthorized,
    Forbidden,
    NotFound,
    BadRequest(String),
    Internal,
}

#[derive(Serialize)]
struct ErrBody { code: &'static str, message: String }

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, message) = match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", "unauthorized".into()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden", "forbidden".into()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found", "not found".into()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, "bad_request", m),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal", "internal error".into()),
        };
        (status, Json(ErrBody { code, message })).into_response()
    }
}

6) 日志与审计：tracing + 绝不记录密钥/身份证/完整 token

记录：who / what / resource_id / ip / user_agent / result

不记录：密码、JWT、TOTP 秘钥、恢复码、证件号全号

axum 常用：

use tower_http::trace::TraceLayer;

let app = Router::new()
  .layer(TraceLayer::new_for_http());


如果你要“事后自证清白”，审计日志要 append-only，最好落到独立存储。

7) 速率限制与爆破防护：tower 层做，而不是业务里 if

登录、发送验证码、导出、搜索接口：都要限速

可用 tower-governor 或者基于 tower::limit

（不贴长代码了）关键是：按 IP + 按账号 两个维度限速。

8) 密码/密钥/敏感配置：用成熟库 + 正确的秘密管理

密码存储：argon2（或 bcrypt）

密钥在内存里：用 secrecy::SecretString，避免 debug 打印

配置：.env 只适合开发；生产上用 KMS/环境变量/secret manager

9) 事务与并发：SeaORM 更新要注意“检查-再更新”的竞态

典型问题：先查权限/余额，再更新，中间被并发修改。

解决：

事务包起来

或用“带条件的 UPDATE”（owner_id/版本号/状态）确保原子性

10) 导出/批量/后台任务：最容易“越权 + 数据外带”

你前面做过“批量导出 PDF/Excel”那类需求，这里必须加：

导出权限单独控制（读数据 ≠ 可导出）

每条记录逐条校验归属/范围

生成结果带审计记录（谁导出了什么范围、多少条）