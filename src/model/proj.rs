use serde::{Deserialize, Deserializer, Serialize};

use crate::model::{
    member::MemberInfoReply,
    moetran::{MtrAllowApplyType, MtrAppliCheckType, MtrRole},
};

#[derive(Debug)]
pub struct ProjCreatePayload {
    pub proj_name: String,
    pub proj_description: Option<String>,

    pub team_id: String,
    pub projset_id: String,

    pub mtr_auth: String,

    pub source_language: String,
    pub target_languages: Vec<String>,

    pub allow_apply_type: MtrAllowApplyType,
    pub application_check_type: MtrAppliCheckType,

    pub default_role: MtrRole,
}

impl<'de> Deserialize<'de> for ProjCreatePayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawProjCreatePayload {
            proj_name: String,
            proj_description: Option<String>,

            team_id: String,
            projset_id: String,

            mtr_auth: String,

            source_language: String,
            target_languages: Vec<String>,

            allow_apply_type: i32,
            application_check_type: i32,

            default_role: String,
        }

        let raw = RawProjCreatePayload::deserialize(deserializer)?;

        // Validate allow_apply_type and map to enum
        let allow_apply_type = match raw.allow_apply_type {
            0 => MtrAllowApplyType::NoApply,
            1 => MtrAllowApplyType::AnyApply,
            2 => MtrAllowApplyType::MemberOnly,
            other => {
                return Err(serde::de::Error::custom(format!(
                    "Invalid allow_apply_type value: {}",
                    other
                )));
            }
        };

        // Validate application_check_type and map to enum
        let application_check_type = match raw.application_check_type {
            0 => MtrAppliCheckType::NonCheck,
            1 => MtrAppliCheckType::AdminCheck,
            other => {
                return Err(serde::de::Error::custom(format!(
                    "Invalid application_check_type value: {}",
                    other
                )));
            }
        };

        // Validate default_role must be one of known role constants and wrap as MtrRole
        let valid_roles = [
            MtrRole::ADMIN,
            MtrRole::PRINCIPAL,
            MtrRole::PROOFREADER,
            MtrRole::TRANSLATOR,
            MtrRole::TYPESETTER,
            MtrRole::INTERN,
        ];

        if !valid_roles.contains(&raw.default_role.as_str()) {
            return Err(serde::de::Error::custom(format!(
                "Invalid default_role value: {}",
                raw.default_role
            )));
        }

        let default_role = if valid_roles.contains(&raw.default_role.as_str()) {
            MtrRole(raw.default_role)
        } else {
            return Err(serde::de::Error::custom(format!(
                "Invalid default_role value: {}",
                raw.default_role
            )));
        };

        Ok(ProjCreatePayload {
            proj_name: raw.proj_name,
            proj_description: raw.proj_description,
            team_id: raw.team_id,
            projset_id: raw.projset_id,
            mtr_auth: raw.mtr_auth,
            source_language: raw.source_language,
            target_languages: raw.target_languages,
            allow_apply_type,
            application_check_type,
            default_role,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct ProjCreateReply {
    pub proj_serial: i32,
    pub projset_index: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ProjStatus {
    NotStarted = 0,
    InProgress = 1,
    Completed = 2,
}

impl Serialize for ProjStatus {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(*self as i32)
    }
}

impl<'de> Deserialize<'de> for ProjStatus {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = i32::deserialize(deserializer)?;

        if value < 0 || value > 2 {
            return Err(serde::de::Error::custom(format!(
                "Invalid ProjStatus value: {}",
                value
            )));
        }

        Ok(ProjStatus::from(value))
    }
}

impl From<i32> for ProjStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => ProjStatus::NotStarted,
            1 => ProjStatus::InProgress,
            2 => ProjStatus::Completed,
            _ => {
                // Unexpected value, default to NotStarted.
                tracing::warn!(
                    "Unexpected ProjStatus value: {}. Defaulting to NotStarted.",
                    value
                );

                ProjStatus::NotStarted
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ProjInfoReply {
    pub proj_id: String,
    pub proj_name: String,
    pub description: Option<String>,

    pub projset_id: String,
    pub projset_serial: i32,
    pub projset_index: i32,

    pub translating_status: ProjStatus,
    pub proofreading_status: ProjStatus,
    pub typesetting_status: ProjStatus,
    pub reviewing_status: ProjStatus,
    pub is_published: bool,

    pub members: Vec<MemberInfoReply>,
}

#[derive(Debug, Deserialize)]
pub struct MarkProjStatusPayload {
    pub proj_id: String,
    /// Available values: "translating", "proofreading",
    /// "typesetting", "reviewing"
    pub status_type: String,
    pub new_status: ProjStatus,
}

#[derive(Debug, Deserialize)]
pub struct SearchProjPayload {
    pub proj_ids: Option<Vec<String>>,

    pub fuzzy_proj_name: Option<String>,

    pub translating_status: Option<ProjStatus>,
    pub proofreading_status: Option<ProjStatus>,
    pub typesetting_status: Option<ProjStatus>,
    pub reviewing_status: Option<ProjStatus>,
    pub is_published: Option<bool>,

    pub member_ids: Option<Vec<String>>,

    pub time_start: Option<i64>,

    pub page: i64,
    pub limit: i64,
}
