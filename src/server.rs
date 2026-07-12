use leptos::prelude::*;

use crate::models::Contact;

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

#[server(ListContacts, "/api")]
pub async fn list_contacts(
    name_query: String,
    email_query: String,
    phone_query: String,
) -> Result<Vec<Contact>, ServerFnError> {
    let state = expect_context::<AppState>();

    let contacts = sqlx::query_as::<_, Contact>(
        r#"
        SELECT id, name, phone, email
        FROM customer
        WHERE name ILIKE $1 ESCAPE '\'
          AND email ILIKE $2 ESCAPE '\'
          AND phone ILIKE $3 ESCAPE '\'
        ORDER BY name
        "#,
    )
    .bind(ilike_pattern(&name_query))
    .bind(ilike_pattern(&email_query))
    .bind(ilike_pattern(&phone_query))
    .fetch_all(&state.pool)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    Ok(contacts)
}

#[server(CreateContact, "/api")]
pub async fn create_contact(
    name: String,
    phone: String,
    email: String,
) -> Result<Contact, ServerFnError> {
    let name = name.trim().to_string();
    let phone = phone.trim().to_string();
    let email = email.trim().to_string();

    if name.is_empty() {
        return Err(ServerFnError::new("Name is required"));
    }

    let contact = Contact::new(name, phone, email);
    let state = expect_context::<AppState>();

    sqlx::query(
        r#"
        INSERT INTO customer (id, name, phone, email)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(contact.id)
    .bind(&contact.name)
    .bind(&contact.phone)
    .bind(&contact.email)
    .execute(&state.pool)
    .await
    .map_err(|err| ServerFnError::new(err.to_string()))?;

    Ok(contact)
}
