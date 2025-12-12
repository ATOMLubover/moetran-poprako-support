use axum::http::StatusCode;
use sqlx::{query, query_as, query_scalar};

use crate::{
    config::AppConfig,
    jwt::JwtCodec,
    model::{
        team::TeamInfoReply,
        user::{SyncTokenReply, SyncUserPayload, UserInfoReply},
    },
    repo::{
        Repo,
        team::TeamBasic,
        user::{NewUser, UserBasic},
    },
    service::{ServiceResult, pass},
};

// /// Hash a plain text password using Argon2.
// fn hash_password(password: &str) -> Result<String, password_hash::Error> {
//     let salt = SaltString::generate(&mut OsRng);
//     let argon2 = Argon2::default();

//     let password_hash = argon2
//         .hash_password(password.as_bytes(), &salt)?
//         .to_string();

//     Ok(password_hash)
// }

// /// Verify a password against a hash.
// fn verify_password(password: &str, password_hash: &str) -> Result<bool, password_hash::Error> {
//     let parsed_hash = PasswordHash::new(password_hash)?;

//     let argon2 = Argon2::default();

//     Ok(argon2
//         .verify_password(password.as_bytes(), &parsed_hash)
//         .is_ok())
// }

/// Synchronize user login by verifying credentials and generating a JWT token.
/// If a user does not exist, it would be created.
pub async fn sync_user(
    args: SyncUserPayload,
    app_config: &AppConfig,
    jwt_codec: &JwtCodec,
    repo: &Repo,
) -> ServiceResult<SyncTokenReply> {
    let mut transac = repo.pool().begin().await?;

    let user_id: Option<String> = query_scalar!(
        r#"
        SELECT f_user_id
        FROM t_user
        WHERE f_user_id = $1
        "#,
        args.user_id
    )
    .fetch_optional(&mut *transac)
    .await?;

    if let Some(user_id) = user_id {
        let token = jwt_codec.encode_token(&user_id, app_config.jwt_exp_seconds)?;

        transac.commit().await?;

        return Ok(pass().with_data(SyncTokenReply { token }));
    }

    // User does not exist, create a new one.

    let new_user = NewUser {
        f_user_id: args.user_id,
        f_username: args.username,
        f_email: args.email,
    };

    query!(
        r#"
        INSERT INTO t_user (f_user_id, f_username, f_email)
        VALUES ($1, $2, $3)
        ON CONFLICT (f_user_id) DO UPDATE
        SET f_username = EXCLUDED.f_username,
            f_email = EXCLUDED.f_email
        "#,
        new_user.f_user_id,
        new_user.f_username,
        new_user.f_email,
    )
    .execute(&mut *transac)
    .await?;

    transac.commit().await?;

    let token = jwt_codec.encode_token(&new_user.f_user_id, app_config.jwt_exp_seconds)?;

    Ok(pass()
        .with_data(SyncTokenReply { token })
        .with_code(StatusCode::CREATED.as_u16()))
}

pub async fn get_user_info(user_id: String, repo: &Repo) -> ServiceResult<UserInfoReply> {
    let user_basic = query_as!(
        UserBasic,
        r#"
        SELECT f_user_id , f_username , f_email 
        FROM t_user
        WHERE f_user_id = $1
        "#,
        user_id
    )
    .fetch_one(&*repo.pool())
    .await?;

    // fetch teams the user belongs to (team id + team name)
    let team_list = query_as!(
        TeamBasic,
        r#"
        SELECT t.f_team_id AS f_team_id, t.f_team_name AS f_team_name
        FROM t_member AS m
        JOIN t_team AS t ON m.f_team_id = t.f_team_id
        WHERE m.f_user_id = $1
        "#,
        user_id
    )
    .fetch_all(&*repo.pool())
    .await?;

    let team_list: Vec<TeamInfoReply> = team_list
        .into_iter()
        .map(|r| TeamInfoReply {
            team_id: r.f_team_id,
            team_name: r.f_team_name,
        })
        .collect();

    let user_info = UserInfoReply {
        user_id: user_basic.f_user_id,
        username: user_basic.f_username,
        email: user_basic.f_email,
        teams: team_list,
    };

    Ok(pass().with_data(user_info))
}
