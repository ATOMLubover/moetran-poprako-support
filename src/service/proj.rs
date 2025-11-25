use std::collections::HashMap;

use axum::http::StatusCode;
use reqwest::{Client, Url};
use sqlx::{ query, query_as};

use crate::{
    cache::{Cache, project::fetch_add_projset_index},
    config::AppConfig,
    crawler::Crawler,
    model::{
        member::MemberInfoReply,
        moetran::{MtrProjectCreatePayload, MtrProjectCreateReply},
        proj::{MarkProjStatusPayload, ProjCreatePayload, ProjCreateReply, ProjInfoReply},
    },
    repo::{Repo, member::MemberPerm, proj::ProjBasic},
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

    // Create the project on Moetran first.
    let mtr_payload = MtrProjectCreatePayload {
        name: args.proj_name,
        intro: args.proj_description,
        project_set: args.projset_id,
        source_language: args.source_language,
        target_languages: args.target_languages,
        allow_apply_type: args.allow_apply_type.as_i32(),
        application_check_type: args.application_check_type.as_i32(),
        default_role: args.default_role.0,
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

    let serial: i32 = sqlx::query_scalar!(
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
        serial,
        projset_index,
        mtr_payload.project_set,
    )
    .execute(&*repo.pool()) 
    .await?;

    Ok(pass()
        .with_code(StatusCode::CREATED.as_u16())
        .with_data(ProjCreateReply {
            proj_serial: serial,
            projset_index,
        }))
}

pub async fn get_projs_by_id(
    proj_ids: Vec<String>,
    repo: &Repo,
) -> ServiceResult<Vec<ProjInfoReply>> {
    if proj_ids.is_empty() {
        return Ok(pass().with_data(Vec::new()));
    }

    // Fetch basic project info.
    let proj_list = query_as!(
        ProjBasic,
        r#"
        SELECT
            f_proj_id,
            f_proj_name,
            f_projset_id,
            f_projset_serial,
            f_projset_index,
            f_translating_status,
            f_proofreading_status,
            f_typesetting_status,
            f_reviewing_status,
            f_is_published
        FROM t_proj
        WHERE f_proj_id = ANY($1)
        "#,
        &proj_ids
    )
    .fetch_all(&*repo.pool())
    .await?;

    // Local struct for joined assignment rows including username.
    struct AssignJoined {
        f_proj_id: String,
        f_member_id: String,
        f_is_admin: bool,
        f_is_translator: bool,
        f_is_proofreader: bool,
        f_is_typesetter: bool,
        f_is_principal: bool,
        f_username: String,
    }

    // Fetch assignments & roles by joining project assignment, projset->team, member & user.
    let assigns = query_as!(
        AssignJoined,
        r#"
        SELECT
            pa.f_proj_id,
            m.f_member_id,
            m.f_is_admin,
            pa.f_is_translator,
            pa.f_is_proofreader,
            pa.f_is_typesetter,
            pa.f_is_principal,
            u.f_username
        FROM t_proj_assgin pa
        JOIN t_proj p ON pa.f_proj_id = p.f_proj_id
        JOIN t_projset ps ON p.f_projset_id = ps.f_projset_id
        JOIN t_member m ON m.f_team_id = ps.f_team_id AND m.f_user_id = pa.f_user_id
        JOIN t_user u ON u.f_user_id = pa.f_user_id
        WHERE pa.f_proj_id = ANY($1)
        "#,
        &proj_ids
    )
    .fetch_all(&*repo.pool())
    .await?;

    let mut proj_map: HashMap<String, ProjInfoReply> = HashMap::new();

    // Initialize map entries.
    for p in proj_list.into_iter() {
        proj_map.insert(
            p.f_proj_id.clone(),
            ProjInfoReply {
                proj_id: p.f_proj_id,
                proj_name: p.f_proj_name,
                description: None, // description not stored yet
                projset_id: p.f_projset_id,
                projset_serial: p.f_projset_serial,
                projset_index: p.f_projset_index,
                translating_status: p.f_translating_status.into(),
                proofreading_status: p.f_proofreading_status.into(),
                typesetting_status: p.f_typesetting_status.into(),
                reviewing_status: p.f_reviewing_status.into(),
                is_published: p.f_is_published,
                members: Vec::new(),
            },
        );
    }

    // Populate members.
    for a in assigns.into_iter() {
        if let Some(info) = proj_map.get_mut(&a.f_proj_id) {
            info.members.push(MemberInfoReply {
                member_id: a.f_member_id,
                username: a.f_username,
                is_admin: a.f_is_admin,
                is_translator: a.f_is_translator,
                is_proofreader: a.f_is_proofreader,
                is_typesetter: a.f_is_typesetter,
                is_principal: a.f_is_principal,
            });
        }
    }

    let mut result: Vec<ProjInfoReply> = proj_map.into_values().collect();
   
    // Optional deterministic order: sort by proj_serial then projset_index.
    result.sort_by(|a, b| a.projset_serial.cmp(&b.projset_serial).then(a.projset_index.cmp(&b.projset_index)));

    Ok(pass().with_data(result))
}

pub async fn mark_proj_status(
    user_id: String,
    args: MarkProjStatusPayload,
    repo: &Repo,
) -> ServiceResult<()> {
    // Check whether the user is a principal of the project.
    let is_principal = sqlx::query_scalar!(
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
            .with_message("Only project principals can update project status.".to_string()));
    }

    // new_status is an enum backed by i32.
    let new_value = args.new_status as i32;

    // Use static SQL per status_type to preserve query! macro type-safety.
    match args.status_type.as_str() {
        "translating" => {
            query!(
                r#"
                UPDATE t_proj
                SET f_translating_status = $1
                WHERE f_proj_id = $2
                "#,
                new_value,
                args.proj_id
            )
            .execute(&*repo.pool())
            .await?;
        }
        "proofreading" => {
            query!(
                r#"
                UPDATE t_proj
                SET f_proofreading_status = $1
                WHERE f_proj_id = $2
                "#,
                new_value,
                args.proj_id
            )
            .execute(&*repo.pool())
            .await?;
        }
        "typesetting" => {
            query!(
                r#"
                UPDATE t_proj
                SET f_typesetting_status = $1
                WHERE f_proj_id = $2
                "#,
                new_value,
                args.proj_id
            )
            .execute(&*repo.pool())
            .await?;
        }
        "reviewing" => {
            query!(
                r#"
                UPDATE t_proj
                SET f_reviewing_status = $1
                WHERE f_proj_id = $2
                "#,
                new_value,
                args.proj_id
            )
            .execute(&*repo.pool())
            .await?;
        }
        _ => {
            return Err(ServiceError::GenericError(
                "Invalid status_type".to_string(),
            ));
        }
    }

    Ok(pass()
        .with_code(StatusCode::NO_CONTENT.as_u16())
        .with_data(()))
}

pub async fn mark_proj_published(
    user_id: String,
    proj_id: String,
    repo: &Repo,
) -> ServiceResult<()> {
    // Check whether the user is a principal of the project.
    let is_principal = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM t_proj_assgin
            WHERE f_proj_id = $1 AND f_user_id = $2 AND f_is_principal = TRUE
        )
        "#,
        proj_id,
        user_id,
    )
    .fetch_one(&*repo.pool())
    .await?
    .unwrap_or(false);

    if !is_principal {
        return Ok(fail()
            .with_code(StatusCode::FORBIDDEN.as_u16())
            .with_message("Only project principals can publish projects.".to_string()));
    }

    // Update the is_published flag to true.
    query!(
        r#"
        UPDATE t_proj
        SET f_is_published = TRUE
        WHERE f_proj_id = $1
        "#,
        proj_id
    )
    .execute(&*repo.pool())
    .await?;

    Ok(pass()
        .with_code(StatusCode::NO_CONTENT.as_u16())
        .with_data(()))
}
