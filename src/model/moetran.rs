use serde::{Deserialize, Serialize};

// ==== Project set DTOs ====

#[derive(Debug, Serialize)]
pub struct MtrProjSetCreatePayload {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct MtrProjSetCreateReply {
    pub message: String,
    #[serde(rename = "project_set")]
    pub projset: MtrProjSetInfoReply,
}

#[derive(Debug, Deserialize)]
pub struct MtrProjSetInfoReply {
    pub id: String,
}

// ==== Project-related enums & DTOs ====

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MtrAppliCheckType {
    NonCheck = 0,
    AdminCheck = 1,
}

impl MtrAppliCheckType {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MtrAllowApplyType {
    NoApply = 0,
    AnyApply = 1,
    MemberOnly = 2,
}

impl MtrAllowApplyType {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtrRole(pub String);

impl MtrRole {
    pub const ADMIN: &'static str = "63d87c24b8bebd75ff934264";
    pub const PRINCIPAL: &'static str = "63d87c24b8bebd75ff934265";
    pub const PROOFREADER: &'static str = "63d87c24b8bebd75ff934266";
    pub const TRANSLATOR: &'static str = "63d87c24b8bebd75ff934267";
    pub const TYPESETTER: &'static str = "63d87c24b8bebd75ff934268";
    pub const INTERN: &'static str = "63d87c24b8bebd75ff934269";

    // pub fn admin() -> Self {
    //     Self(Self::ADMIN.to_string())
    // }
    // pub fn supervisor() -> Self {
    //     Self(Self::PRINCIPAL.to_string())
    // }
    // pub fn proofreader() -> Self {
    //     Self(Self::PROOFREADER.to_string())
    // }
    // pub fn translator() -> Self {
    //     Self(Self::TRANSLATOR.to_string())
    // }
    // pub fn embedder() -> Self {
    //     Self(Self::TYPESETTER.to_string())
    // }
    // pub fn intern() -> Self {
    //     Self(Self::INTERN.to_string())
    // }
}

/// 语言代码常量
pub mod mtr_lang {
    pub const JAPANESE: &str = "ja"; // 日语
    pub const ZH_CN: &str = "zh-CN"; // 简体中文
    pub const ZH_TW: &str = "zh-TW"; // 繁体中文
    pub const KOREAN: &str = "ko"; // 韩语
    pub const ENGLISH: &str = "en"; // 英语
}

#[derive(Debug, Serialize)]
pub struct MtrProjectCreatePayload {
    pub name: String,
    pub intro: Option<String>,
    pub source_language: String,
    pub target_languages: Vec<String>,
    pub allow_apply_type: i32,
    pub application_check_type: i32,
    pub default_role: String,
    pub project_set: String,
}

#[derive(Debug, Deserialize)]
pub struct MtrProjectCreateReply {
    pub message: String,
    #[serde(rename = "project")]
    pub project: MtrProjectInfoReply,
}

#[derive(Debug, Deserialize)]
pub struct MtrProjectInfoReply {
    pub id: String,
}
