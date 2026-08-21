use leptos::prelude::*;
use uuid::Uuid;

use crate::models::{ContactCard, ContactDetail, ContactKind, Person};

#[cfg(feature = "ssr")]
use crate::models::ContactRow;

#[cfg(feature = "ssr")]
use crate::state::AppState;

#[cfg(feature = "ssr")]
fn ilike_pattern(query: &str) -> String {
    if query.is_empty() {
        return "%".to_string();
    }

    let escaped = query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
    format!("%{escaped}%")
}

#[cfg(feature = "ssr")]
async fn load_person(
    pool: &sqlx::PgPool,
    id: Uuid,
) -> Result<Option<Person>, ServerFnError> {
    sqlx::query_as::<_, Person>(
        r#"
        SELECT id, name, phone, email, life_stage, family_id
        FROM person
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))
}

#[cfg(feature = "ssr")]
async fn load_family_members(
    pool: &sqlx::PgPool,
    family_id: Uuid,
) -> Result<Vec<Person>, ServerFnError> {
    sqlx::query_as::<_, Person>(
        r#"
        SELECT id, name, phone, email, life_stage, family_id
        FROM person
        WHERE family_id = $1
          AND deleted_at IS NULL
        ORDER BY
            CASE life_stage
                WHEN 'parent' THEN 0
                WHEN 'older' THEN 1
                WHEN 'young_adult' THEN 2
                WHEN 'child' THEN 3
                ELSE 4
            END,
            name
        "#,
    )
    .bind(family_id)
    .fetch_all(pool)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))
}

#[cfg(feature = "ssr")]
async fn load_contact_row(
    pool: &sqlx::PgPool,
    id: Uuid,
) -> Result<ContactRow, ServerFnError> {
    sqlx::query_as::<_, ContactRow>(
        r#"
        SELECT id, person_id, family_id
        FROM contact
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?
    .ok_or_else(|| ServerFnError::new("Contact not found"))
}

#[cfg(feature = "ssr")]
async fn build_contact_detail(
    pool: &sqlx::PgPool,
    row: ContactRow,
) -> Result<ContactDetail, ServerFnError> {
    if let Some(person_id) = row.person_id {
        let person = load_person(pool, person_id)
            .await?
            .ok_or_else(|| ServerFnError::new("Person not found"))?;
        Ok(ContactDetail::Person {
            contact_id: row.id,
            person,
        })
    } else if let Some(family_id) = row.family_id {
        let members = load_family_members(pool, family_id).await?;
        Ok(ContactDetail::Family {
            contact_id: row.id,
            family_id,
            members,
        })
    } else {
        Err(ServerFnError::new("Contact has no target"))
    }
}

#[cfg(feature = "ssr")]
async fn insert_person(pool: &sqlx::PgPool, person: &Person) -> Result<(), ServerFnError> {
    sqlx::query(
        r#"
        INSERT INTO person (
            id, name, phone, email, life_stage, family_id,
            created_at, modified_at, deleted_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW(), NULL)
        "#,
    )
    .bind(person.id)
    .bind(&person.name)
    .bind(&person.phone)
    .bind(&person.email)
    .bind(person.life_stage)
    .bind(person.family_id)
    .execute(pool)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(())
}

#[cfg(feature = "ssr")]
async fn soft_delete_person(pool: &sqlx::PgPool, person_id: Uuid) -> Result<(), ServerFnError> {
    sqlx::query(
        r#"
        UPDATE person
        SET deleted_at = NOW(), modified_at = NOW()
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(person_id)
    .execute(pool)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(())
}

#[cfg(feature = "ssr")]
async fn soft_delete_family(pool: &sqlx::PgPool, family_id: Uuid) -> Result<(), ServerFnError> {
    sqlx::query(
        r#"
        UPDATE person
        SET deleted_at = NOW(), modified_at = NOW()
        WHERE family_id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(family_id)
    .execute(pool)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    sqlx::query(
        r#"
        UPDATE family
        SET deleted_at = NOW(), modified_at = NOW()
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(family_id)
    .execute(pool)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(())
}

#[cfg(feature = "ssr")]
async fn soft_delete_contact_row(pool: &sqlx::PgPool, contact_id: Uuid) -> Result<(), ServerFnError> {
    sqlx::query(
        r#"
        UPDATE contact
        SET deleted_at = NOW(), modified_at = NOW()
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(contact_id)
    .execute(pool)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;
    Ok(())
}

#[server(ListContacts, "/api")]
pub async fn list_contacts(name_query: String) -> Result<Vec<ContactCard>, ServerFnError> {
    let state = expect_context::<AppState>();
    let pattern = ilike_pattern(&name_query);

    let rows = sqlx::query_as::<_, ContactRow>(
        r#"
        SELECT DISTINCT c.id, c.person_id, c.family_id
        FROM contact c
        LEFT JOIN person person_target
            ON person_target.id = c.person_id
           AND person_target.deleted_at IS NULL
        LEFT JOIN person family_member
            ON family_member.family_id = c.family_id
           AND family_member.deleted_at IS NULL
        WHERE c.deleted_at IS NULL
          AND (
            (
                c.person_id IS NOT NULL
                AND person_target.name <> ''
                AND person_target.name ILIKE $1 ESCAPE '\'
            )
            OR (
                c.family_id IS NOT NULL
                AND family_member.name <> ''
                AND family_member.name ILIKE $1 ESCAPE '\'
            )
        )
        ORDER BY c.id
        "#,
    )
    .bind(pattern)
    .fetch_all(&state.pool)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    let mut cards = Vec::new();
    for row in rows {
        let detail = build_contact_detail(&state.pool, row).await?;
        let card = match detail {
            ContactDetail::Person {
                contact_id,
                person,
            } => ContactCard::from_person(contact_id, &person),
            ContactDetail::Family {
                contact_id,
                members,
                ..
            } => ContactCard::from_family(contact_id, &members),
        };
        if card.has_title() {
            cards.push(card);
        }
    }

    cards.sort_by(|left, right| {
        left.title
            .to_lowercase()
            .cmp(&right.title.to_lowercase())
            .then_with(|| left.id.cmp(&right.id))
    });

    Ok(cards)
}

#[server(GetContactDetail, "/api")]
pub async fn get_contact_detail(id: Uuid) -> Result<ContactDetail, ServerFnError> {
    let state = expect_context::<AppState>();
    let row = load_contact_row(&state.pool, id).await?;
    build_contact_detail(&state.pool, row).await
}

#[server(CreateContact, "/api")]
pub async fn create_contact(kind: ContactKind) -> Result<ContactDetail, ServerFnError> {
    let state = expect_context::<AppState>();
    let contact_id = Uuid::new_v4();

    match kind {
        ContactKind::Person => {
            let person = Person::draft(None);
            insert_person(&state.pool, &person).await?;
            sqlx::query(
                r#"
                INSERT INTO contact (
                    id, person_id, family_id, created_at, modified_at, deleted_at
                )
                VALUES ($1, $2, NULL, NOW(), NOW(), NULL)
                "#,
            )
            .bind(contact_id)
            .bind(person.id)
            .execute(&state.pool)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;

            Ok(ContactDetail::Person {
                contact_id,
                person,
            })
        }
        ContactKind::Family => {
            let family_id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO family (id, created_at, modified_at, deleted_at)
                VALUES ($1, NOW(), NOW(), NULL)
                "#,
            )
            .bind(family_id)
            .execute(&state.pool)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;

            let person = Person::draft(Some(family_id));
            insert_person(&state.pool, &person).await?;
            sqlx::query(
                r#"
                INSERT INTO contact (
                    id, person_id, family_id, created_at, modified_at, deleted_at
                )
                VALUES ($1, NULL, $2, NOW(), NOW(), NULL)
                "#,
            )
            .bind(contact_id)
            .bind(family_id)
            .execute(&state.pool)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;

            Ok(ContactDetail::Family {
                contact_id,
                family_id,
                members: vec![person],
            })
        }
    }
}

#[server(UpdatePerson, "/api")]
pub async fn update_person(
    id: Uuid,
    name: String,
    phone: String,
    email: String,
    life_stage: Option<crate::models::LifeStage>,
) -> Result<Person, ServerFnError> {
    let name = name.trim().to_string();
    let phone = phone.trim().to_string();
    let email = email.trim().to_string();
    let state = expect_context::<AppState>();

    let result = sqlx::query(
        r#"
        UPDATE person
        SET name = $2,
            phone = $3,
            email = $4,
            life_stage = $5,
            modified_at = NOW()
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .bind(&name)
    .bind(&phone)
    .bind(&email)
    .bind(life_stage)
    .execute(&state.pool)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Person not found"));
    }

    load_person(&state.pool, id)
        .await?
        .ok_or_else(|| ServerFnError::new("Person not found"))
}

#[server(AddFamilyMember, "/api")]
pub async fn add_family_member(family_id: Uuid) -> Result<Person, ServerFnError> {
    let state = expect_context::<AppState>();

    let family_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM family
            WHERE id = $1
              AND deleted_at IS NULL
        )
        "#,
    )
    .bind(family_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    if !family_exists {
        return Err(ServerFnError::new("Family not found"));
    }

    let person = Person::draft(Some(family_id));
    insert_person(&state.pool, &person).await?;

    sqlx::query(
        r#"
        UPDATE family
        SET modified_at = NOW()
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(family_id)
    .execute(&state.pool)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    Ok(person)
}

#[server(DeleteFamilyMember, "/api")]
pub async fn delete_family_member(person_id: Uuid) -> Result<(), ServerFnError> {
    let state = expect_context::<AppState>();
    let person = load_person(&state.pool, person_id)
        .await?
        .ok_or_else(|| ServerFnError::new("Person not found"))?;

    let Some(family_id) = person.family_id else {
        return Err(ServerFnError::new("Person is not a family member"));
    };

    soft_delete_person(&state.pool, person_id).await?;

    sqlx::query(
        r#"
        UPDATE family
        SET modified_at = NOW()
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(family_id)
    .execute(&state.pool)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    Ok(())
}

#[server(ConvertContactToFamily, "/api")]
pub async fn convert_contact_to_family(contact_id: Uuid) -> Result<ContactDetail, ServerFnError> {
    let state = expect_context::<AppState>();
    let row = load_contact_row(&state.pool, contact_id).await?;
    let person_id = row
        .person_id
        .ok_or_else(|| ServerFnError::new("Contact is not a person contact"))?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

    let family_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO family (id, created_at, modified_at, deleted_at)
        VALUES ($1, NOW(), NOW(), NULL)
        "#,
    )
    .bind(family_id)
    .execute(&mut *tx)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    sqlx::query(
        r#"
        UPDATE person
        SET family_id = $2, modified_at = NOW()
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(person_id)
    .bind(family_id)
    .execute(&mut *tx)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    sqlx::query(
        r#"
        UPDATE contact
        SET person_id = NULL, family_id = $2, modified_at = NOW()
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(contact_id)
    .bind(family_id)
    .execute(&mut *tx)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    tx.commit()
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

    let row = load_contact_row(&state.pool, contact_id).await?;
    build_contact_detail(&state.pool, row).await
}

#[server(ConvertContactToPerson, "/api")]
pub async fn convert_contact_to_person(contact_id: Uuid) -> Result<ContactDetail, ServerFnError> {
    let state = expect_context::<AppState>();
    let row = load_contact_row(&state.pool, contact_id).await?;
    let family_id = row
        .family_id
        .ok_or_else(|| ServerFnError::new("Contact is not a family contact"))?;

    let members = load_family_members(&state.pool, family_id).await?;
    if members.len() != 1 {
        return Err(ServerFnError::new(
            "Remove extra family members before converting to a person",
        ));
    }
    let person_id = members[0].id;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

    sqlx::query(
        r#"
        UPDATE person
        SET family_id = NULL, modified_at = NOW()
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(person_id)
    .execute(&mut *tx)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    sqlx::query(
        r#"
        UPDATE contact
        SET person_id = $2, family_id = NULL, modified_at = NOW()
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(contact_id)
    .bind(person_id)
    .execute(&mut *tx)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    sqlx::query(
        r#"
        UPDATE family
        SET deleted_at = NOW(), modified_at = NOW()
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(family_id)
    .execute(&mut *tx)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    tx.commit()
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

    let row = load_contact_row(&state.pool, contact_id).await?;
    build_contact_detail(&state.pool, row).await
}

#[server(DeleteContact, "/api")]
pub async fn delete_contact(id: Uuid) -> Result<(), ServerFnError> {
    let state = expect_context::<AppState>();
    let row = load_contact_row(&state.pool, id).await?;

    soft_delete_contact_row(&state.pool, id).await?;

    if let Some(person_id) = row.person_id {
        soft_delete_person(&state.pool, person_id).await?;
    } else if let Some(family_id) = row.family_id {
        soft_delete_family(&state.pool, family_id).await?;
    }

    Ok(())
}
