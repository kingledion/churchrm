use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "ssr",
    derive(sqlx::Type),
    sqlx(type_name = "life_stage", rename_all = "snake_case")
)]
pub enum LifeStage {
    Child,
    YoungAdult,
    Parent,
    Older,
}

impl LifeStage {
    pub const ALL: [Self; 4] = [Self::Child, Self::YoungAdult, Self::Parent, Self::Older];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Child => "child",
            Self::YoungAdult => "young_adult",
            Self::Parent => "parent",
            Self::Older => "older",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Child => "Child",
            Self::YoungAdult => "Young adult",
            Self::Parent => "Parent",
            Self::Older => "Older",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Child => "Child in the household",
            Self::YoungAdult => "Young adult / no kids yet",
            Self::Parent => "Parent with children at home",
            Self::Older => "Older, or adult children",
        }
    }

    pub fn is_child(self) -> bool {
        matches!(self, Self::Child)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContactKind {
    Person,
    Family,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Person {
    pub id: Uuid,
    pub name: String,
    pub phone: String,
    pub email: String,
    pub life_stage: Option<LifeStage>,
    pub family_id: Option<Uuid>,
}

impl Person {
    pub fn draft(family_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            phone: String::new(),
            email: String::new(),
            life_stage: None,
            family_id,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.name.trim().is_empty()
            && self.phone.trim().is_empty()
            && self.email.trim().is_empty()
            && self.life_stage.is_none()
    }

    pub fn shows_contact_info(&self) -> bool {
        !self.life_stage.is_some_and(LifeStage::is_child)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ContactRow {
    pub id: Uuid,
    pub person_id: Option<Uuid>,
    pub family_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactCard {
    pub id: Uuid,
    pub kind: ContactKind,
    pub title: String,
    pub child_names: Vec<String>,
    pub emails: Vec<String>,
    pub phones: Vec<String>,
    pub life_stage: Option<LifeStage>,
}

impl ContactCard {
    pub fn from_person(contact_id: Uuid, person: &Person) -> Self {
        Self {
            id: contact_id,
            kind: ContactKind::Person,
            title: person.name.clone(),
            child_names: Vec::new(),
            emails: non_empty(&person.email),
            phones: non_empty(&person.phone),
            life_stage: person.life_stage,
        }
    }

    pub fn from_family(contact_id: Uuid, members: &[Person]) -> Self {
        let adults: Vec<&Person> = members
            .iter()
            .filter(|person| person.shows_contact_info())
            .collect();
        let children: Vec<&Person> = members
            .iter()
            .filter(|person| !person.shows_contact_info())
            .collect();

        let title = adults
            .iter()
            .map(|person| person.name.trim())
            .filter(|name| !name.is_empty())
            .collect::<Vec<_>>()
            .join(" & ");

        let child_names = children
            .iter()
            .map(|person| person.name.trim())
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .collect();

        let emails = adults
            .iter()
            .flat_map(|person| non_empty(&person.email))
            .collect();
        let phones = adults
            .iter()
            .flat_map(|person| non_empty(&person.phone))
            .collect();

        let life_stage = adults.iter().find_map(|person| person.life_stage);

        Self {
            id: contact_id,
            kind: ContactKind::Family,
            title,
            child_names,
            emails,
            phones,
            life_stage,
        }
    }

    pub fn has_title(&self) -> bool {
        !self.title.trim().is_empty() || !self.child_names.is_empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContactDetail {
    Person {
        contact_id: Uuid,
        person: Person,
    },
    Family {
        contact_id: Uuid,
        family_id: Uuid,
        members: Vec<Person>,
    },
}

impl ContactDetail {
    pub fn contact_id(&self) -> Uuid {
        match self {
            Self::Person { contact_id, .. } | Self::Family { contact_id, .. } => *contact_id,
        }
    }

    pub fn kind(&self) -> ContactKind {
        match self {
            Self::Person { .. } => ContactKind::Person,
            Self::Family { .. } => ContactKind::Family,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Person { person, .. } => person.is_empty(),
            Self::Family { members, .. } => members.iter().all(Person::is_empty),
        }
    }
}

fn non_empty(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Vec::new()
    } else {
        vec![trimmed.to_string()]
    }
}
