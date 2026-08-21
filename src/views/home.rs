use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_navigate};

use crate::components::{ContactInfoIcon, ContactInfoStatus, LifeStageIcon};
use crate::models::ContactCard;
use crate::server::list_contacts;

#[component]
pub fn HomePage() -> impl IntoView {
    let name_query = RwSignal::new(String::new());
    let navigate = use_navigate();

    let contacts = Resource::new(
        move || name_query.get(),
        |name| async move { list_contacts(name).await.unwrap_or_default() },
    );

    view! {
        <section class="page">
            <div class="page-toolbar">
                <div class="contact-list-toolbar">
                    <label class="sr-only" for="filter-name">"Filter by name"</label>
                    <input
                        id="filter-name"
                        class="name-filter"
                        type="search"
                        placeholder="Filter by name"
                        prop:value=move || name_query.get()
                        on:input=move |ev| name_query.set(event_target_value(&ev))
                    />
                </div>
                <button
                    type="button"
                    class="button button-primary"
                    on:click=move |_| navigate("/contacts/new", Default::default())
                >
                    "New contact"
                </button>
            </div>

            <Transition fallback=|| view! { <p class="empty-state">"Loading contacts..."</p> }>
                {move || Suspend::new(async move {
                    let rows = contacts.await;
                    let count = rows.len();
                    view! {
                        <ContactCardList contacts=rows/>
                        <p class="result-count">
                            {count}
                            {if count == 1 { " contact" } else { " contacts" }}
                        </p>
                    }
                })}
            </Transition>
        </section>
    }
}

#[component]
fn ContactCardList(contacts: Vec<ContactCard>) -> impl IntoView {
    if contacts.is_empty() {
        return view! {
            <p class="empty-state">"No contacts match your search."</p>
        }
        .into_any();
    }

    view! {
        <div class="contact-card-list">
            {contacts
                .into_iter()
                .map(|card| {
                    let href = format!("/contacts/{}", card.id);
                    let status = ContactInfoStatus::from_card(&card);
                    let life_stage = card.life_stage;
                    let child_names = card.child_names.join(", ");
                    let hide_children = child_names.is_empty();
                    view! {
                        <div class="contact-card">
                            <A href=href attr:class="contact-card__link">
                                <span class="contact-card__text">
                                    <span class="contact-card__name">{card.title}</span>
                                    <span
                                        class="contact-card__children muted"
                                        prop:hidden=hide_children
                                    >
                                        {child_names}
                                    </span>
                                </span>
                                <span class="contact-card__icons">
                                    <ContactInfoIcon status=status/>
                                    <LifeStageIcon life_stage=life_stage/>
                                </span>
                            </A>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
    .into_any()
}
