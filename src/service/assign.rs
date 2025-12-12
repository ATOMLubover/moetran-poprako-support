use axum::http::StatusCode;
use reqwest::Client;
use sqlx::{query, query_scalar};
use url::Url;

use crate::{
    config::AppConfig,
    crawler::Crawler,
    model::{
        assign::{ProjAssignInfoReply, ProjAssignPayload},
        moetran::{MtrInviteMemberReply, MtrInviteMemberRequest, MtrRole},
    },
    repo::Repo,
    service::{ServiceError, ServiceResult, fail, pass},
};

/// Call external Moetran API to invite/assign a member to a project.
/// This is modeled after the Go client's InviteMemberToProject.
async fn assign_mtr_member(
    client: &Client,
    base_url: &str,
    project_id: &str,
    auth: &str,
    payload: MtrInviteMemberRequest,
) -> Result<(), ServiceError> {
    let mut url = Url::parse(base_url)?;

    // base_url 应该类似 https://api.moetran.com/v1
    // 这里拼出 /projects/{project_id}/invitations
    url.path_segments_mut()
        .map_err(|_| {
            ServiceError::GenericError("Invalid base_url for path modifications".to_string())
        })?
        .push("projects")
        .push(project_id)
        .push("invitations");

    // 允许 auth 传入原始 token 或带 Bearer 前缀
    let token = auth.strip_prefix("Bearer ").unwrap_or(auth);

    tracing::debug!(
        ?url,
        "Inviting member on Moetran with payload: {:?}",
        payload
    );

    let response = client
        .post(url)
        .json(&payload)
        .bearer_auth(token)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<could not read body>".to_string());

        tracing::warn!(
            "Moetran invite member failed: status = {}, body = {}",
            status,
            body
        );

        return Err(ServiceError::GenericError(format!(
            "Moetran invite member error: {} - {}",
            status, body
        )));
    }

    // 如果需要，可以解析 reply；目前只验证成功状态即可。
    let _: MtrInviteMemberReply = response.json().await?;

    Ok(())
}

pub async fn assign_member(
    crawler: &Crawler,
    config: &AppConfig,
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
    #[allow(dead_code)]
    struct MemberBasic {
        f_member_id: String,
        f_user_id: String,
        f_team_id: String,
        f_is_translator: bool,
        f_is_proofreader: bool,
        f_is_typesetter: bool,
        f_is_redrawer: bool,
    }

    let member = sqlx::query_as!(
        MemberBasic,
        r#"
        SELECT
            f_member_id,
            f_user_id,
            f_team_id,
            f_is_translator,
            f_is_proofreader,
            f_is_typesetter,
            f_is_redrawer
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
    if args.is_redrawer && !member.f_is_redrawer {
        return Err(ServiceError::GenericError(
            "Member is not a redrawer".to_string(),
        ));
    }

    // Before calling Moetran, check whether this (proj_id, user_id)
    // already exists in the local assignment table. If it does,
    // skip the external invite (treat as idempotent) but still
    // perform the local upsert below to ensure role flags are current.

    let already_assigned: bool = query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM t_proj_assgin WHERE f_proj_id = $1 AND f_user_id = $2
        )
        "#,
        args.proj_id,
        member.f_user_id
    )
    .fetch_one(&*repo.pool())
    .await?
    .unwrap_or(false);

    if !already_assigned {
        let mtr_payload = MtrInviteMemberRequest {
            user_id: member.f_user_id.clone(),
            role_id: MtrRole::PROOFREADER.to_string(), // Default to translator for now.
            message: String::new(),                    // empty invitation message by default
        };

        assign_mtr_member(
            crawler.client(),
            &config.mtr_base_url,
            &args.proj_id,
            &args.mtr_auth,
            mtr_payload,
        )
        .await?;
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
            f_is_redrawer,
            f_is_principal
        )
        VALUES (
            gen_random_uuid()::text,
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7
        )
        ON CONFLICT (f_proj_id, f_user_id)
        DO UPDATE SET
            f_is_translator = EXCLUDED.f_is_translator,
            f_is_proofreader = EXCLUDED.f_is_proofreader,
            f_is_typesetter = EXCLUDED.f_is_typesetter,
            f_is_redrawer = EXCLUDED.f_is_redrawer
        "#,
        args.proj_id,
        member.f_user_id,
        args.is_translator,
        args.is_proofreader,
        args.is_typesetter,
        args.is_redrawer,
        false,
    )
    .execute(repo.pool())
    .await?;

    Ok(pass()
        .with_code(StatusCode::NO_CONTENT.as_u16())
        .with_data(()))
}

// Helper: sanitize pagination (private, placed above the public function)
fn sanitize_pagination(page: i64, limit: i64) -> (i64, i64, i64) {
    let page = page.max(1);
    let limit = limit.max(1);
    let offset = (page - 1) * limit;
    (page, limit, offset)
}

pub async fn get_assigns(
    time_start: i64,
    page: i64,
    limit: i64,
    repo: &Repo,
) -> ServiceResult<Vec<ProjAssignInfoReply>> {
    // Local joined struct defined within service function.
    struct AssignJoined {
        f_proj_id: String,
        f_proj_name: String,
        f_projset_serial: i32,
        f_projset_index: i32,
        f_user_id: String,
        f_username: String,
        f_is_translator: bool,
        f_is_proofreader: bool,
        f_is_typesetter: bool,
        f_is_redrawer: bool,
        f_is_principal: bool,
        f_created_at: time::OffsetDateTime,
    }

    let timestamp = time::OffsetDateTime::from_unix_timestamp(time_start)
        .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);

    let (_page, limit, offset) = sanitize_pagination(page, limit);

    let rows = sqlx::query_as!(
        AssignJoined,
        r#"
        SELECT
            pa.f_proj_id,
            p.f_proj_name,
            ps.f_projset_serial,
            p.f_projset_index,
            pa.f_user_id as "f_user_id",
            u.f_username,
            pa.f_is_translator,
            pa.f_is_proofreader,
            pa.f_is_typesetter,
            pa.f_is_redrawer,
            pa.f_is_principal,
            pa.f_created_at
        FROM t_proj_assgin pa
        JOIN t_proj p ON pa.f_proj_id = p.f_proj_id
        JOIN t_projset ps ON p.f_projset_id = ps.f_projset_id
        JOIN t_user u ON pa.f_user_id = u.f_user_id
        WHERE pa.f_created_at > $1
        ORDER BY pa.f_created_at ASC
        LIMIT $2 OFFSET $3
        "#,
        timestamp,
        limit,
        offset
    )
    .fetch_all(&*repo.pool())
    .await?;

    let replies = rows
        .into_iter()
        .map(|a| ProjAssignInfoReply {
            proj_id: a.f_proj_id,
            proj_name: a.f_proj_name,
            projset_serial: a.f_projset_serial,
            projset_index: a.f_projset_index,
            member_id: a.f_user_id,
            username: a.f_username,
            is_translator: a.f_is_translator,
            is_proofreader: a.f_is_proofreader,
            is_typesetter: a.f_is_typesetter,
            is_redrawer: a.f_is_redrawer,
            is_principal: a.f_is_principal,
            updated_at: a.f_created_at,
        })
        .collect::<Vec<_>>();

    Ok(pass().with_data(replies))
}
