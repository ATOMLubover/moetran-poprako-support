use axum::http::StatusCode;
use sqlx::query_as;

use crate::{
    model::member::{MemberAbstract, MemberInfoReply},
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
        f_is_principal: bool,
        f_username: String,
    }

    let member = query_as!(
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
        member_id: member.f_member_id,
        username: member.f_username,
        is_admin: member.f_is_admin,
        is_translator: member.f_is_translator,
        is_proofreader: member.f_is_proofreader,
        is_typesetter: member.f_is_typesetter,
        is_principal: member.f_is_principal,
    };

    Ok(pass().with_data(member_info))
}

pub async fn pick_members_by_position(
    team_id: String,
    position: String,
    page: i64,
    limit: i64,
    repo: &Repo,
) -> ServiceResult<Vec<MemberAbstract>> {
    // Basic pagination sanitization.
    let page = if page < 1 { 1 } else { page };
    let limit = if limit < 1 { 10 } else { limit };

    let offset = (page - 1) * limit;

    struct MemberJoined {
        f_member_id: String,
        f_user_id: String,
        f_team_id: String,
        f_is_admin: bool,
        f_is_translator: bool,
        f_is_proofreader: bool,
        f_is_typesetter: bool,
        f_is_principal: bool,
        f_username: String,
    }

    let members = match position.as_str() {
        "translator" => {
            sqlx::query_as!(
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
                m.f_is_principal,
                u.f_username
            FROM t_member m
            JOIN t_user u ON m.f_user_id = u.f_user_id
            WHERE m.f_team_id = $1 AND m.f_is_translator = TRUE
            OFFSET $2 LIMIT $3
            "#,
                team_id,
                offset,
                limit
            )
            .fetch_all(&*repo.pool())
            .await?
        }
        "proofreader" => {
            sqlx::query_as!(
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
                m.f_is_principal,
                u.f_username
            FROM t_member m
            JOIN t_user u ON m.f_user_id = u.f_user_id
            WHERE m.f_team_id = $1 AND m.f_is_proofreader = TRUE
            OFFSET $2 LIMIT $3
            "#,
                team_id,
                offset,
                limit
            )
            .fetch_all(&*repo.pool())
            .await?
        }
        "typesetter" => {
            sqlx::query_as!(
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
                m.f_is_principal,
                u.f_username
            FROM t_member m
            JOIN t_user u ON m.f_user_id = u.f_user_id
            WHERE m.f_team_id = $1 AND m.f_is_typesetter = TRUE
            OFFSET $2 LIMIT $3
            "#,
                team_id,
                offset,
                limit
            )
            .fetch_all(&*repo.pool())
            .await?
        }
        "principal" => {
            sqlx::query_as!(
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
                m.f_is_principal,
                u.f_username
            FROM t_member m
            JOIN t_user u ON m.f_user_id = u.f_user_id
            WHERE m.f_team_id = $1 AND m.f_is_principal = TRUE
            OFFSET $2 LIMIT $3
            "#,
                team_id,
                offset,
                limit
            )
            .fetch_all(&*repo.pool())
            .await?
        }
        _ => return Err(ServiceError::GenericError("Invalid position".to_string())),
    };

    let abstracts = members
        .into_iter()
        .map(|m| MemberAbstract {
            member_id: m.f_member_id,
            username: m.f_username,
        })
        .collect::<Vec<_>>();

    Ok(pass().with_data(abstracts))
}
