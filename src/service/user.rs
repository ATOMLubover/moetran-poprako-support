use crate::{
    model::user::{LoginToken, LoginUser},
    repo::Repo,
    service::ServiceValue,
};

pub async fn login_user(args: LoginUser, repo: Repo) -> ServiceValue<LoginToken> {
    todo!()
}
