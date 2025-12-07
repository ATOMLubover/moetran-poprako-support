use std::collections::HashMap;

use axum::http::StatusCode;
use reqwest::{Client, Url};
use sqlx::{query, query_as};

use crate::{
    cache::{Cache, project::fetch_add_projset_index},
    config::AppConfig,
    crawler::Crawler,
    model::{
        member::MemberInfoReply,
        moetran::{MtrProjectCreatePayload, MtrProjectCreateReply},
        proj::{
            MarkProjStatusPayload, ProjCreatePayload, ProjCreateReply, ProjInfoReply,
            SearchProjPayload,
        },
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
    let mut url = Url::parse(base_url)?;

    // Append path segments safely using the url crate's path_segments_mut API
    // instead of concatenating strings. This avoids Url::join's surprising
    // replacement behavior when joining multiple times.
    url.path_segments_mut()
        .map_err(|_| {
            ServiceError::GenericError("Invalid base_url for path modifications".to_string())
        })?
        .push("teams")
        .push(team_id)
        .push("projects");

    // Normalize token: allow caller to pass either the raw token or
    // the full header value "Bearer <token>". Avoid sending
    // "Bearer Bearer <token>" to the external API.
    let token = auth.strip_prefix("Bearer ").unwrap_or(auth);

    tracing::debug!(
        ?url,
        ?base_url,
        "Creating Moetran project with payload: {:?}",
        payload
    );

    let response = client
        .post(url)
        .json(payload)
        .bearer_auth(token)
        .send()
        .await?;

    // If the external API returned a non-success status, capture
    // the response body for logging and return a readable error.
    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<could not read body>".to_string());
        tracing::warn!(
            "Moetran project create failed: status = {}, body = {}",
            status,
            body
        );
        return Err(ServiceError::GenericError(format!(
            "Moetran API error: {} - {}",
            status, body
        )));
    }

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

    // Fetch and increment the projset index from cache.
    let projset_index = fetch_add_projset_index(&args.projset_id, cache).await?;

    let serial: i32 = sqlx::query_scalar!(
        r#"
        SELECT f_projset_serial
        FROM t_projset
        WHERE f_team_id = $1 AND f_projset_id = $2
        "#,
        args.team_id,
        args.projset_id
    )
    .fetch_one(&*repo.pool())
    .await?;

    let proj_name = format!("【{}-{}】{}", serial, projset_index, args.proj_name);

    // Create the project on Moetran first.
    let mtr_payload = MtrProjectCreatePayload {
        name: proj_name,
        intro: args.proj_description,
        project_set: args.projset_id,
        source_language: args.source_language,
        target_languages: args.target_languages,
        allow_apply_type: 3,
        application_check_type: 1,
        default_role: args.default_role.0,
    };

    tracing::debug!("Creating Moetran project with payload: {:?}", mtr_payload);

    let project_id = create_mtr_project(
        crawler.client(),
        &config.mtr_base_url,
        &args.team_id,
        &args.mtr_auth,
        &mtr_payload,
    )
    .await?;

    tracing::debug!("Moetran project created with ID: {}", project_id);

    // Create the project record in our database.
    query!(
        r#"
        INSERT INTO t_proj (f_proj_id, f_proj_name, f_projset_serial, f_projset_index, f_projset_id)
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

    // Ensure the creating user is assigned to the project as a principal.
    // This upsert mirrors the behavior in `service::assign::assign_member`.
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
            f_is_principal = EXCLUDED.f_is_principal
        "#,
        project_id,
        user_id,
        false, // is_translator
        false, // is_proofreader
        false, // is_typesetter
        false, // is_redrawer
        true,  // is_principal => creator becomes principal
    )
    .execute(&*repo.pool())
    .await?;

    Ok(pass()
        .with_code(StatusCode::CREATED.as_u16())
        .with_data(ProjCreateReply {
            proj_id: project_id,
            proj_serial: serial,
            projset_index,
        }))
}

pub async fn search_projs(
    args: SearchProjPayload,
    repo: &Repo,
) -> ServiceResult<Vec<ProjInfoReply>> {
    // If proj_ids is provided, fetch by ids directly.
    match args.proj_ids {
        Some(proj_ids) if !proj_ids.is_empty() => {
            return get_projs_by_id(proj_ids, repo).await;
        }
        _ => {}
    }

    // Base SQL selecting from t_proj and left joining assignments and users to build members list.
    let mut sql = String::from(
        r#"
        SELECT
            p.f_proj_id,
            p.f_proj_name,
            p.f_projset_id,
            p.f_projset_serial,
            p.f_projset_index,
            p.f_translating_status,
            p.f_proofreading_status,
            p.f_typesetting_status,
            p.f_reviewing_status,
            p.f_is_published,
                m.f_member_id,
                pa.f_user_id,
            m.f_is_admin,
            pa.f_is_translator,
            pa.f_is_proofreader,
            pa.f_is_typesetter,
            pa.f_is_redrawer,
            pa.f_is_principal,
            u.f_username
        FROM t_proj p
        LEFT JOIN t_proj_assgin pa ON pa.f_proj_id = p.f_proj_id
        LEFT JOIN t_member m ON m.f_user_id = pa.f_user_id
        LEFT JOIN t_user u ON u.f_user_id = pa.f_user_id
        "#,
    );

    let mut conditions: Vec<String> = Vec::new();

    // We'll bind parameters in order using a simple enum to keep types.
    enum Param {
        Str(String),
        I32(i32),
        I64(i64),
        Bool(bool),
        StrArray(Vec<String>),
    }

    let mut bind_params: Vec<Param> = Vec::new();

    // We manually push conditions and keep track of placeholder index.
    let mut idx: i32 = 1;

    // Fuzzy project name (ILIKE on f_proj_name).
    if let Some(name) = &args.fuzzy_proj_name {
        conditions.push(format!("p.f_proj_name ILIKE ${}", idx));

        idx += 1;

        bind_params.push(Param::Str(format!("%{}%", name)));
    }

    // Filter by projset ids if provided.
    if let Some(projset_ids) = &args.projset_ids {
        if !projset_ids.is_empty() {
            conditions.push(format!("p.f_projset_id = ANY(${})", idx));

            idx += 1;

            bind_params.push(Param::StrArray(projset_ids.clone()));
        }
    }

    // time_start filters projects created at or after the given unix timestamp (seconds).
    if let Some(ts) = args.time_start {
        conditions.push(format!("p.f_created_at >= to_timestamp(${})", idx));

        idx += 1;

        bind_params.push(Param::I64(ts));
    }

    // Status filters
    if let Some(s) = args.translating_status {
        conditions.push(format!("p.f_translating_status = ${}", idx));

        idx += 1;

        let v = s as i32;

        bind_params.push(Param::I32(v));
    }
    if let Some(s) = args.proofreading_status {
        conditions.push(format!("p.f_proofreading_status = ${}", idx));

        idx += 1;

        let v = s as i32;

        bind_params.push(Param::I32(v));
    }
    if let Some(s) = args.typesetting_status {
        conditions.push(format!("p.f_typesetting_status = ${}", idx));

        idx += 1;

        let v = s as i32;

        bind_params.push(Param::I32(v));
    }
    if let Some(s) = args.reviewing_status {
        conditions.push(format!("p.f_reviewing_status = ${}", idx));

        idx += 1;

        let v = s as i32;

        bind_params.push(Param::I32(v));
    }

    if let Some(published) = args.is_published {
        conditions.push(format!("p.f_is_published = ${}", idx));

        idx += 1;

        bind_params.push(Param::Bool(published));
    }

    // Filter by member ids if provided.
    if let Some(member_ids) = &args.member_ids {
        if !member_ids.is_empty() {
            // filter by member IDs stored on t_member.f_member_id
            conditions.push(format!("m.f_member_id = ANY(${})", idx));

            idx += 1;

            bind_params.push(Param::StrArray(member_ids.clone()));
        }
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    // Order and pagination.
    sql.push_str(" ORDER BY p.f_projset_serial, p.f_projset_index LIMIT $");
    sql.push_str(&idx.to_string());

    idx += 1;

    sql.push_str(" OFFSET $");
    sql.push_str(&idx.to_string());

    let mut query = sqlx::query_as::<_, Row>(&sql);

    // Bind dynamic params in order with concrete types.
    for param in bind_params {
        query = match param {
            Param::Str(v) => query.bind(v),
            Param::I32(v) => query.bind(v),
            Param::I64(v) => query.bind(v),
            Param::Bool(v) => query.bind(v),
            Param::StrArray(v) => query.bind(v),
        };
    }

    let page = args.page.unwrap_or(1).max(1);
    let limit = args.limit.unwrap_or(10).max(1);
    let offset = (page - 1) * limit;

    query = query.bind(limit).bind(offset);

    // Row mapping struct
    #[derive(sqlx::FromRow)]
    struct Row {
        f_proj_id: String,
        f_proj_name: String,
        f_projset_id: String,
        f_projset_serial: i32,
        f_projset_index: i32,
        f_translating_status: i32,
        f_proofreading_status: i32,
        f_typesetting_status: i32,
        f_reviewing_status: i32,
        f_is_published: bool,
        f_member_id: Option<String>,
        f_user_id: Option<String>,
        f_is_admin: Option<bool>,
        f_is_translator: Option<bool>,
        f_is_proofreader: Option<bool>,
        f_is_typesetter: Option<bool>,
        f_is_redrawer: Option<bool>,
        f_is_principal: Option<bool>,
        f_username: Option<String>,
    }

    let rows: Vec<Row> = query.fetch_all(&*repo.pool()).await?;

    let mut proj_map: HashMap<String, ProjInfoReply> = HashMap::new();

    for r in rows.into_iter() {
        let entry = proj_map
            .entry(r.f_proj_id.clone())
            .or_insert_with(|| ProjInfoReply {
                proj_id: r.f_proj_id.clone(),
                proj_name: r.f_proj_name.clone(),
                description: None,
                projset_id: r.f_projset_id.clone(),
                projset_serial: r.f_projset_serial,
                projset_index: r.f_projset_index,
                translating_status: r.f_translating_status.into(),
                proofreading_status: r.f_proofreading_status.into(),
                typesetting_status: r.f_typesetting_status.into(),
                reviewing_status: r.f_reviewing_status.into(),
                is_published: r.f_is_published,
                members: Vec::new(),
            });

        if let (Some(member_id), Some(username)) = (r.f_member_id, r.f_username) {
            entry.members.push(MemberInfoReply {
                user_id: r.f_user_id.unwrap_or_default(),
                member_id,
                username,
                is_admin: r.f_is_admin.unwrap_or(false),
                is_translator: r.f_is_translator.unwrap_or(false),
                is_proofreader: r.f_is_proofreader.unwrap_or(false),
                is_typesetter: r.f_is_typesetter.unwrap_or(false),
                is_redrawer: r.f_is_redrawer.unwrap_or(false),
                is_principal: r.f_is_principal.unwrap_or(false),
            });
        }
    }

    let mut result: Vec<ProjInfoReply> = proj_map.into_values().collect();
    result.sort_by(|a, b| {
        a.projset_serial
            .cmp(&b.projset_serial)
            .then(a.projset_index.cmp(&b.projset_index))
    });

    Ok(pass().with_data(result))
}

async fn get_projs_by_id(proj_ids: Vec<String>, repo: &Repo) -> ServiceResult<Vec<ProjInfoReply>> {
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
        f_user_id: String,
        f_is_admin: bool,
        f_is_translator: bool,
        f_is_proofreader: bool,
        f_is_typesetter: bool,
        f_is_redrawer: bool,
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
            pa.f_user_id,
            m.f_is_admin,
            pa.f_is_translator,
            pa.f_is_proofreader,
            pa.f_is_typesetter,
            pa.f_is_redrawer,
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
                user_id: a.f_user_id,
                member_id: a.f_member_id,
                username: a.f_username,
                is_admin: a.f_is_admin,
                is_translator: a.f_is_translator,
                is_proofreader: a.f_is_proofreader,
                is_typesetter: a.f_is_typesetter,
                is_redrawer: a.f_is_redrawer,
                is_principal: a.f_is_principal,
            });
        }
    }

    let mut result: Vec<ProjInfoReply> = proj_map.into_values().collect();

    // Optional deterministic order: sort by proj_serial then projset_index.
    result.sort_by(|a, b| {
        a.projset_serial
            .cmp(&b.projset_serial)
            .then(a.projset_index.cmp(&b.projset_index))
    });

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
