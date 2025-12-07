use crate::http::result::HttpResult;
use crate::model::notify::UpdateNotifyReply;

pub async fn check_update() -> HttpResult<UpdateNotifyReply> {
    HttpResult::new(
        axum::http::StatusCode::OK,
        None,
        Some(UpdateNotifyReply { has_update: false }),
    )
}
