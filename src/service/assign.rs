use axum::http::StatusCode;
use sqlx::{query, query_scalar};

use crate::{
    model::assign::ProjAssignPayload,
    repo::Repo,
    service::{ServiceError, ServiceResult, fail, pass},
};

pub async fn assign_member(
    user_id: String,
    args: ProjAssignPayload,
    repo: &Repo,
) -> ServiceResult<()> {
    // Check whether the operator is a principal of this project.
    // We treat "principal" as a per-project role stored in t_proj_assgin.
    let is_principal = query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM t_proj_assgin
            WHERE f_proj_id = $1 AND f_user_id = $2 AND f_is_principal = TRUE
        )
        "#,
        args.proj_id,
        user_id,
    )
    .fetch_one(&*repo.pool())
    .await?
    .unwrap_or(false);

    if !is_principal {
        return Ok(fail()
            .with_code(StatusCode::FORBIDDEN.as_u16())
            .with_message("Only project principals can assign members.".to_string()));
    }

    // Check whether the target member belongs to this project's team
    // and is capable of the assigned roles.

    // Find the team of the project via projset.
    let proj_team: String = query_scalar!(
        r#"
        SELECT ps.f_team_id
        FROM t_proj p
        JOIN t_projset ps ON p.f_projset_id = ps.f_projset_id
        WHERE p.f_proj_id = $1
        "#,
        args.proj_id
    )
    .fetch_one(&*repo.pool())
    .await?;

    // Load the member record within that team.
    struct MemberBasic {
        f_member_id: String,
        f_user_id: String,
        f_team_id: String,
        f_is_admin: bool,
        f_is_translator: bool,
        f_is_proofreader: bool,
        f_is_typesetter: bool,
        f_is_principal: bool,
    }

    let member = sqlx::query_as!(
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
        WHERE f_member_id = $1 AND f_team_id = $2
        "#,
        args.member_id,
        proj_team
    )
    .fetch_optional(&*repo.pool())
    .await?;

    let member = match member {
        Some(m) => m,
        None => {
            return Err(ServiceError::GenericError(
                "Member not found in project team".to_string(),
            ));
        }
    };

    // Capability checks: if a role is requested, ensure member has that role flag.
    if args.is_translator && !member.f_is_translator {
        return Err(ServiceError::GenericError(
            "Member is not a translator".to_string(),
        ));
    }
    if args.is_proofreader && !member.f_is_proofreader {
        return Err(ServiceError::GenericError(
            "Member is not a proofreader".to_string(),
        ));
    }
    if args.is_typesetter && !member.f_is_typesetter {
        return Err(ServiceError::GenericError(
            "Member is not a typesetter".to_string(),
        ));
    }
    if args.is_principal && !member.f_is_principal {
        return Err(ServiceError::GenericError(
            "Member is not a principal".to_string(),
        ));
    }

    // Upsert the assignment record into t_proj_assgin.
    // Unique key: (f_proj_id, f_user_id). We use ON CONFLICT to update.

    query!(
        r#"
        INSERT INTO t_proj_assgin (
            f_proj_assgin_id,
            f_proj_id,
            f_user_id,
            f_is_translator,
            f_is_proofreader,
            f_is_typesetter,
            f_is_principal
        )
        VALUES (
            gen_random_uuid()::text,
            $1,
            $2,
            $3,
            $4,
            $5,
            $6
        )
        ON CONFLICT (f_proj_id, f_user_id)
        DO UPDATE SET
            f_is_translator = EXCLUDED.f_is_translator,
            f_is_proofreader = EXCLUDED.f_is_proofreader,
            f_is_typesetter = EXCLUDED.f_is_typesetter,
            f_is_principal = EXCLUDED.f_is_principal
        "#,
        args.proj_id,
        member.f_user_id,
        args.is_translator,
        args.is_proofreader,
        args.is_typesetter,
        args.is_principal,
    )
    .execute(&*repo.pool())
    .await?;

    Ok(pass()
        .with_code(StatusCode::NO_CONTENT.as_u16())
        .with_data(()))
}
