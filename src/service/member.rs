use axum::http::StatusCode;
use sqlx::query_as;

use crate::{
    model::member::MemberInfoReply,
    repo::{Repo, member::MemberBasic},
    service::{ServiceResult, fail},
};

pub async fn get_member_info(
    user_id: String,
    team_id: String,
    repo: &Repo,
) -> ServiceResult<MemberInfoReply> {
    let member = query_as!(
        MemberBasic,
        r#"
        SELECT 
            f_member_id,
            f_user_id,
            f_team_id,
            f_is_admin,
            f_is_translator,
            f_is_proofreader,
            f_is_typesetter,
            f_is_principal
        FROM t_member
        WHERE f_team_id = $1
        "#,
        team_id
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

    if user_id != member.f_user_id {
        return Ok(fail().with_code(StatusCode::FORBIDDEN.as_u16()));
    }

    let member_info = MemberInfoReply {
        member_id: member.f_member_id,
        is_admin: member.f_is_admin,
        is_translator: member.f_is_translator,
        is_proofreader: member.f_is_proofreader,
        is_typesetter: member.f_is_typesetter,
        is_principal: member.f_is_principal,
    };

    Ok(crate::service::pass().with_data(member_info))
}
