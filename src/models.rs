use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Household / life stage for a contact.
///
/// - `Young`: single or does not yet have kids
/// - `HasKids`: has children who are still in the house
/// - `Older`: older, or all children are adults
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "ssr",
    derive(sqlx::Type),
    sqlx(type_name = "life_stage", rename_all = "snake_case")
)]
pub enum LifeStage {
    Young,
    HasKids,
    Older,
}

impl LifeStage {
    pub const ALL: [Self; 3] = [Self::Young, Self::HasKids, Self::Older];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Young => "young",
            Self::HasKids => "has_kids",
            Self::Older => "older",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "young" => Some(Self::Young),
            "has_kids" => Some(Self::HasKids),
            "older" => Some(Self::Older),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Young => "Young",
            Self::HasKids => "With children",
            Self::Older => "Older",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Young => "Single or does not yet have kids",
            Self::HasKids => "Has children still living at home",
            Self::Older => "Older, or all children are adults",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Contact {
    pub id: Uuid,
    pub name: String,
    pub phone: String,
    pub email: String,
    pub life_stage: Option<LifeStage>,
}

impl Contact {
    pub fn new(name: String, phone: String, email: String, life_stage: Option<LifeStage>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            phone,
            email,
            life_stage,
        }
    }
}
