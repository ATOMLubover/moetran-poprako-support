use axum::http::StatusCode;
use reqwest::{Client, Url};
use sqlx::{ query, query_as};

use crate::{
    cache::{Cache, project::fetch_add_projset_index},
    config::AppConfig,
    crawler::Crawler,
    model::{
        moetran::{MtrProjectCreatePayload, MtrProjectCreateReply},
        project::{ProjCreatePayload, ProjCreateReply},
    },
    repo::{Repo, member::MemberPerm},
    service::{ServiceError, ServiceResult, fail, pass},
};

/// Call external Moetran API to create a project and return its id.
async fn create_mtr_project(
    client: &Client,
    base_url: &str,
    team_id: &str,
    auth: &str,
    payload: &MtrProjectCreatePayload,
) -> Result<String, ServiceError> {
    let url = Url::parse(base_url)?;

    let url = url.join("teams")?.join(team_id)?.join("projects")?;

    let response = client
        .post(url)
        .json(payload)
        .bearer_auth(auth)
        .send()
        .await?;

    let response = response.error_for_status()?;

    let reply: MtrProjectCreateReply = response.json().await?;

    Ok(reply.project.id)
}

pub async fn create_proj(
    user_id: String,
    args: ProjCreatePayload,
    config: &AppConfig,
    crawler: &Crawler,
    cache: &Cache,
    repo: &Repo,
) -> ServiceResult<ProjCreateReply> {
    // Check whether the user is the admin of the team.
    let user = sqlx::query_as!(
        MemberPerm,
        r#"
        SELECT
            f_user_id,
            f_team_id,
            f_is_admin
        FROM t_member
        WHERE f_team_id = $1 AND f_user_id = $2
        "#,
        args.team_id,
        user_id
    )
    .fetch_optional(&*repo.pool())
    .await?;

    if user.is_none() || !user.as_ref().unwrap().f_is_admin {
        return Ok(fail()
            .with_code(StatusCode::FORBIDDEN.as_u16())
            .with_message("Only team admins can create project sets.".to_string()));
    }

    // Create the project on Moetran first.
    let mtr_payload = MtrProjectCreatePayload {
        name: args.proj_name,
        intro: args.proj_description,
        project_set: args.projset_id,
        source_language: args.source_language,
        target_languages: args.target_languages,
        allow_apply_type: args.allow_apply_type,
        application_check_type: args.application_check_type,
        default_role: args.default_role,
    };

    let project_id = create_mtr_project(
        crawler.client(),
        &config.mtr_base_url,
        &args.team_id,
        &args.mtr_auth,
        &mtr_payload,
    )
    .await?;

    // Fetch and increment the projset index from cache.
    let projset_index = fetch_add_projset_index(&mtr_payload.project_set, cache).await?;

    struct ProjsetSerial {
        f_projset_serial: i32,
    }

    let serial = query_as!(
        ProjsetSerial,
        r#"
        SELECT f_projset_serial 
        FROM t_projset
        WHERE f_team_id = $1 AND f_projset_id = $2
        "#,
        args.team_id,
        mtr_payload.project_set
    )
    .fetch_one(&*repo.pool())
    .await?;

    // Create the project record in our database.
    query!(
        r#"
        INSERT INTO t_proj (f_proj_id, f_proj_name,  f_projset_serial, f_projset_index, f_projset_id)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        project_id,
        mtr_payload.name,
        serial.f_projset_serial,
        projset_index,
        mtr_payload.project_set,
    )
    .execute(&*repo.pool()) 
    .await?;

    Ok(pass()
        .with_code(StatusCode::CREATED.as_u16())
        .with_data(ProjCreateReply {
            proj_serial: serial.f_projset_serial,
            projset_index,
        }))
}
