use axum::http::StatusCode;
use sqlx::{Row, query};
use time::OffsetDateTime;

use crate::{
    model::member::{GetActiveMemberReply, MemberInfoReply, SearchMemberPayload},
    repo::Repo,
    service::{ServiceError, ServiceResult, fail, pass},
};

pub async fn get_member_info(
    user_id: String,
    team_id: String,
    repo: &Repo,
) -> ServiceResult<MemberInfoReply> {
    struct MemberJoined {
        f_member_id: String,
        f_user_id: String,
        f_team_id: String,
        f_is_admin: bool,
        f_is_translator: bool,
        f_is_proofreader: bool,
        f_is_typesetter: bool,
        f_is_redrawer: bool,
        f_is_principal: bool,
        f_username: String,
    }

    let member = sqlx::query_as!(
        MemberJoined,
        r#"
        SELECT 
            m.f_member_id,
            m.f_user_id,
            m.f_team_id,
            m.f_is_admin,
            m.f_is_translator,
            m.f_is_proofreader,
            m.f_is_typesetter,
            m.f_is_redrawer,
            m.f_is_principal,
            u.f_username
        FROM t_member m
        JOIN t_user u ON m.f_user_id = u.f_user_id
        WHERE m.f_team_id = $1 AND m.f_user_id = $2
        "#,
        team_id,
        user_id
    )
    .fetch_optional(&*repo.pool())
    .await?;

    let member = match member {
        Some(m) => m,
        None => {
            return Ok(fail()
                .with_code(StatusCode::NOT_FOUND.as_u16())
                .with_message("Member not found.".to_string()));
        }
    };

    let member_info = MemberInfoReply {
        user_id: member.f_user_id,
        member_id: member.f_member_id,
        username: member.f_username,
        is_admin: member.f_is_admin,
        is_translator: member.f_is_translator,
        is_proofreader: member.f_is_proofreader,
        is_typesetter: member.f_is_typesetter,
        is_redrawer: member.f_is_redrawer,
        is_principal: member.f_is_principal,
    };

    Ok(pass().with_data(member_info))
}

pub async fn search_members(
    args: SearchMemberPayload,
    repo: &Repo,
) -> ServiceResult<Vec<GetActiveMemberReply>> {
    // Extract and sanitize pagination.
    let page = args.page.unwrap_or(1).max(1);
    let limit = args.limit.unwrap_or(10).max(1);
    let offset = (page - 1) * limit;

    // Build SQL conditionally similar to search_projs: parameter placeholders increment.
    let mut sql = String::from(
        r#"
        SELECT
            m.f_member_id,
            m.f_user_id,
            m.f_team_id,
            m.f_is_admin,
            m.f_is_translator,
            m.f_is_proofreader,
            m.f_is_typesetter,
            m.f_is_redrawer,
            m.f_is_principal,
            u.f_username,
            MAX(pa.f_created_at) as last_active
        FROM t_member m
        JOIN t_user u ON m.f_user_id = u.f_user_id
        LEFT JOIN t_proj_assgin pa ON pa.f_user_id = m.f_user_id
        LEFT JOIN t_proj p ON p.f_proj_id = pa.f_proj_id
        LEFT JOIN t_projset ps ON ps.f_projset_id = p.f_projset_id
        "#,
    );

    let mut conditions: Vec<String> = Vec::new();
    enum Param {
        Str(String),
        I64(i64),
    }
    let mut bind_params: Vec<Param> = Vec::new();
    let mut idx: i32 = 1;

    // team_id is required in the model payload
    conditions.push(format!("m.f_team_id = ${}", idx));
    conditions.push(format!("(ps.f_team_id = ${} OR ps.f_team_id IS NULL)", idx));
    idx += 1;
    bind_params.push(Param::Str(args.team_id));

    // position filters
    if let Some(pos) = args.position.as_deref() {
        match pos {
            "translator" => {
                conditions.push(format!("m.f_is_translator = TRUE"));
            }
            "proofreader" => {
                conditions.push(format!("m.f_is_proofreader = TRUE"));
            }
            "typesetter" => {
                conditions.push(format!("m.f_is_typesetter = TRUE"));
            }
            "redrawer" => {
                conditions.push(format!("m.f_is_redrawer = TRUE"));
            }
            "principal" => {
                conditions.push(format!("m.f_is_principal = TRUE"));
            }
            _ => return Err(ServiceError::GenericError("Invalid position".to_string())),
        }
    }

    // fuzzy_name
    if let Some(name) = args.fuzzy_name {
        conditions.push(format!("u.f_username ILIKE ${}", idx));
        idx += 1;
        bind_params.push(Param::Str(format!("%{}%", name)));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" GROUP BY m.f_member_id, m.f_user_id, m.f_team_id, m.f_is_admin, ");
    sql.push_str("m.f_is_translator, m.f_is_proofreader, m.f_is_typesetter, ");
    sql.push_str("m.f_is_redrawer, m.f_is_principal, u.f_username");
    sql.push_str(" ORDER BY last_active DESC NULLS LAST, m.f_member_id LIMIT $");
    sql.push_str(&idx.to_string());
    idx += 1;
    sql.push_str(" OFFSET $");
    sql.push_str(&idx.to_string());

    let mut query = query(&sql);
    for param in bind_params {
        query = match param {
            Param::Str(v) => query.bind(v),
            Param::I64(v) => query.bind(v),
        };
    }

    query = query.bind(limit).bind(offset);

    let rows = query.fetch_all(&*repo.pool()).await?;

    let members = rows
        .into_iter()
        .map(|row| {
            let member_id: String = row.try_get("f_member_id").unwrap_or_default();
            let user_id: String = row.try_get("f_user_id").unwrap_or_default();
            let username: String = row.try_get("f_username").unwrap_or_default();
            let is_admin: bool = row.try_get("f_is_admin").unwrap_or(false);
            let is_translator: bool = row.try_get("f_is_translator").unwrap_or(false);
            let is_proofreader: bool = row.try_get("f_is_proofreader").unwrap_or(false);
            let is_typesetter: bool = row.try_get("f_is_typesetter").unwrap_or(false);
            let is_redrawer: bool = row.try_get("f_is_redrawer").unwrap_or(false);
            let is_principal: bool = row.try_get("f_is_principal").unwrap_or(false);
            let last_active: Option<OffsetDateTime> = row.try_get("last_active").ok();

            GetActiveMemberReply {
                member_id,
                user_id,
                username,
                is_admin,
                is_translator,
                is_proofreader,
                is_typesetter,
                is_redrawer,
                is_principal,
                last_active,
            }
        })
        .collect::<Vec<_>>();

    Ok(pass().with_data(members))
}

pub async fn get_active_members(
    team_id: String,
    page: i64,
    limit: i64,
    repo: &Repo,
) -> ServiceResult<Vec<GetActiveMemberReply>> {
    // Extract and sanitize pagination.
    let page = page.max(1);
    let limit = limit.max(1);
    let offset = (page - 1) * limit;

    // Query to get all members with their last assignment activity in the team
    let sql = r#"
        SELECT 
            m.f_member_id,
            m.f_user_id,
            m.f_team_id,
            m.f_is_admin,
            m.f_is_translator,
            m.f_is_proofreader,
            m.f_is_typesetter,
            m.f_is_redrawer,
            m.f_is_principal,
            u.f_username,
            MAX(pa.f_created_at) as last_active
        FROM t_member m
        JOIN t_user u ON m.f_user_id = u.f_user_id
        LEFT JOIN t_proj_assgin pa ON pa.f_user_id = m.f_user_id
        LEFT JOIN t_proj p ON p.f_proj_id = pa.f_proj_id
        LEFT JOIN t_projset ps ON ps.f_projset_id = p.f_projset_id AND ps.f_team_id = $1
        WHERE m.f_team_id = $1
        GROUP BY m.f_member_id, m.f_user_id, m.f_team_id, m.f_is_admin, 
                 m.f_is_translator, m.f_is_proofreader, m.f_is_typesetter,
                 m.f_is_redrawer, m.f_is_principal, u.f_username
        ORDER BY last_active DESC NULLS LAST, m.f_member_id
        LIMIT $2 OFFSET $3
        "#;

    let rows = query(sql)
        .bind(&team_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&*repo.pool())
        .await?;

    let members = rows
        .into_iter()
        .map(|row| {
            let member_id: String = row.try_get("f_member_id").unwrap_or_default();
            let user_id: String = row.try_get("f_user_id").unwrap_or_default();
            let username: String = row.try_get("f_username").unwrap_or_default();
            let is_admin: bool = row.try_get("f_is_admin").unwrap_or(false);
            let is_translator: bool = row.try_get("f_is_translator").unwrap_or(false);
            let is_proofreader: bool = row.try_get("f_is_proofreader").unwrap_or(false);
            let is_typesetter: bool = row.try_get("f_is_typesetter").unwrap_or(false);
            let is_redrawer: bool = row.try_get("f_is_redrawer").unwrap_or(false);
            let is_principal: bool = row.try_get("f_is_principal").unwrap_or(false);
            let last_active: Option<OffsetDateTime> = row.try_get("last_active").ok();

            GetActiveMemberReply {
                member_id,
                user_id,
                username,
                is_admin,
                is_translator,
                is_proofreader,
                is_typesetter,
                is_redrawer,
                is_principal,
                last_active,
            }
        })
        .collect::<Vec<_>>();

    Ok(pass().with_data(members))
}
