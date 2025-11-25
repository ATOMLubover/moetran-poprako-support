use axum::http::StatusCode;
use reqwest::{Client, Url};
use sqlx::query;

use crate::{
    cache::{
        Cache,
        projset::{fetch_add_projset_serial, init_projset_index},
    },
    config::AppConfig,
    crawler::Crawler,
    model::{
        moetran::{MtrProjSetCreatePayload, MtrProjSetCreateReply},
        projset::{ProjSetCreatePayload, ProjSetCreateReply, ProjSetInfoReply},
    },
    repo::{Repo, member::MemberPerm, projset::ProjSetBasic},
    service::{ServiceError, ServiceResult, fail, pass},
};

/// Call external Moetran API to create a project set and return its id.
async fn create_mtr_projset(
    client: &Client,
    base_url: &str,
    team_id: &str,
    auth: &str,
    name: &str,
) -> Result<String, ServiceError> {
    let url = Url::parse(base_url)?;

    let url = url.join("teams")?.join(team_id)?.join("project-sets")?;

    let mtr_payload = MtrProjSetCreatePayload {
        name: name.to_owned(),
    };

    let response = client
        .post(url)
        .json(&mtr_payload)
        .bearer_auth(auth)
        .send()
        .await?;

    let response = response.error_for_status()?;

    let reply: MtrProjSetCreateReply = response.json().await?;

    Ok(reply.projset.id)
}

pub async fn create_projset(
    user_id: String,
    args: ProjSetCreatePayload,
    config: &AppConfig,
    crawler: &Crawler,
    cache: &Cache,
    repo: &Repo,
) -> ServiceResult<ProjSetCreateReply> {
    // Check whether the user is the admin of the team.
    let member_perm = sqlx::query_as!(
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

    if member_perm.is_none() || !member_perm.as_ref().unwrap().f_is_admin {
        return Ok(fail()
            .with_code(StatusCode::FORBIDDEN.as_u16())
            .with_message("Only team admins can create project sets.".to_string()));
    }

    // Try to create a project set in Moetran.
    let projset_id = create_mtr_projset(
        crawler.client(),
        &config.mtr_base_url,
        &args.team_id,
        &args.mtr_token,
        &args.projset_name,
    )
    .await?;

    // Fetch and add the team project set serial.
    let projset_serial = fetch_add_projset_serial(&args.team_id, cache).await?;

    if projset_serial <= 0 {
        return Err(ServiceError::GenericError(
            "Failed to fetch a valid project set serial.".to_string(),
        ));
    }

    // Create a new project set in repo, whose
    // project set serial is the fetched serial.
    let mut trx = repo.pool().begin().await?;

    query!(
        r#"
        INSERT INTO t_projset (f_projset_id, f_team_id, f_projset_name, f_projset_description, f_projset_serial)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        projset_id,
        args.team_id,
        args.projset_name,
        args.projset_description,
        projset_serial,
    )
    .execute(&mut *trx)
    .await?;

    // Initialize the project set index in cache.
    // SAFETY: As no project can be inserted to the target project set if its index counter
    // creation fails, we have to ensure this step keeps coherent with the repo transaction.
    let successful = init_projset_index(&projset_id, cache).await?;

    if !successful {
        return Err(ServiceError::GenericError(
            "Failed to initialize the project set index in cache.".to_string(),
        ));
    }

    // Commit the transaction.
    trx.commit().await?;

    Ok(pass()
        .with_code(StatusCode::CREATED.as_u16())
        .with_data(ProjSetCreateReply { projset_serial }))
}

pub async fn get_projsets_by_id(
    projset_ids: Vec<String>,
    repo: &Repo,
) -> ServiceResult<Vec<ProjSetInfoReply>> {
    let projset_list = sqlx::query_as!(
        ProjSetBasic,
        r#"
        SELECT 
            f_projset_id,
            f_projset_name,
            f_projset_description,
            f_projset_serial,
            f_team_id
        FROM t_projset
        WHERE f_projset_id = ANY($1)
        "#,
        &projset_ids
    )
    .fetch_all(&*repo.pool())
    .await?;

    let projsets: Vec<ProjSetInfoReply> = projset_list
        .into_iter()
        .map(|ps| ProjSetInfoReply {
            projset_id: ps.f_projset_id,
            projset_name: ps.f_projset_name,
            projset_description: ps.f_projset_description,
            projset_serial: ps.f_projset_serial,
            team_id: ps.f_team_id,
        })
        .collect();

    Ok(pass().with_data(projsets))
}
