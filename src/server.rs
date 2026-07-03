use leptos::prelude::*;

use crate::models::Contact;

#[cfg(feature = "ssr")]
use crate::state::AppState;

#[cfg(feature = "ssr")]
fn matches_query(value: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    value.to_lowercase().contains(&query.to_lowercase())
}

#[server(ListContacts, "/api")]
pub async fn list_contacts(
    name_query: String,
    email_query: String,
    phone_query: String,
) -> Result<Vec<Contact>, ServerFnError> {
    let state = expect_context::<AppState>();
    let contacts = state
        .contacts
        .read()
        .map_err(|_| ServerFnError::new("Failed to read contacts"))?;

    let filtered = contacts
        .iter()
        .filter(|contact| {
            matches_query(&contact.name, &name_query)
                && matches_query(&contact.email, &email_query)
                && matches_query(&contact.phone, &phone_query)
        })
        .cloned()
        .collect();

    Ok(filtered)
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
    let mut contacts = state
        .contacts
        .write()
        .map_err(|_| ServerFnError::new("Failed to write contacts"))?;
    contacts.push(contact.clone());

    Ok(contact)
}
