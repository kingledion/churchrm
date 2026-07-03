use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{A, Route, Router, Routes},
    hooks::use_navigate,
    StaticSegment,
};

use crate::models::Contact;
use crate::server::{create_contact, list_contacts};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/churchrm.css"/>
        <Title text="ChurchRM"/>
        <Router>
            <div class="app-shell">
                <header class="app-header">
                    <h1 class="app-title">"ChurchRM"</h1>
                    <p class="app-subtitle">"Parish contact directory"</p>
                </header>
                <main class="app-main">
                    <Routes fallback=NotFound>
                        <Route path=StaticSegment("") view=HomePage/>
                        <Route path=StaticSegment("new") view=NewContactPage/>
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let name_query = RwSignal::new(String::new());
    let email_query = RwSignal::new(String::new());
    let phone_query = RwSignal::new(String::new());

    let contacts = Resource::new(
        move || (name_query.get(), email_query.get(), phone_query.get()),
        |(name, email, phone)| async move {
            list_contacts(name, email, phone)
                .await
                .unwrap_or_default()
        },
    );

    view! {
        <section class="page">
            <div class="page-toolbar">
                <h2 class="page-heading">"Contacts"</h2>
                <A href="/new" attr:class="button button-primary">"New contact"</A>
            </div>

            <div class="table-scroll">
                <table class="contacts-table">
                    <thead>
                        <tr class="contacts-table__header-row">
                            <th>
                                <ColumnHeaderInput
                                    id="filter-name"
                                    label="Name"
                                    query=name_query
                                />
                            </th>
                            <th>
                                <ColumnHeaderInput
                                    id="filter-email"
                                    label="Email"
                                    query=email_query
                                />
                            </th>
                            <th>
                                <ColumnHeaderInput
                                    id="filter-phone"
                                    label="Phone"
                                    query=phone_query
                                />
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        <Transition fallback=|| view! { <ContactRows contacts=Vec::new() loading=true/> }>
                            {move || {
                                contacts
                                    .get()
                                    .map(|rows| view! { <ContactRows contacts=rows/> })
                            }}
                        </Transition>
                    </tbody>
                </table>
            </div>

            <Suspense fallback=|| ()>
                {move || {
                    contacts.get().map(|rows| {
                        let count = rows.len();
                        view! {
                            <p class="result-count">
                                {count}
                                {if count == 1 { " contact" } else { " contacts" }}
                            </p>
                        }
                    })
                }}
            </Suspense>
        </section>
    }
}

#[component]
fn ColumnHeaderInput(id: &'static str, label: &'static str, query: RwSignal<String>) -> impl IntoView {
    view! {
        <label class="sr-only" for=id>{label}</label>
        <input
            id=id
            class="column-header-input"
            type="text"
            placeholder=label
            prop:value=move || query.get()
            on:input=move |ev| query.set(event_target_value(&ev))
        />
    }
}

#[component]
fn ContactRows(contacts: Vec<Contact>, #[prop(optional)] loading: bool) -> impl IntoView {
    if loading {
        return view! {
            <tr>
                <td colspan="3" class="empty-state">"Loading contacts..."</td>
            </tr>
        }
        .into_any();
    }

    if contacts.is_empty() {
        return view! {
            <tr>
                <td colspan="3" class="empty-state">"No contacts match your search."</td>
            </tr>
        }
        .into_any();
    }

    contacts
        .into_iter()
        .map(|contact| {
            view! {
                <tr>
                    <td data-label="Name">{contact.name}</td>
                    <td data-label="Email">
                        {if contact.email.is_empty() {
                            view! { <span class="muted">"—"</span> }.into_any()
                        } else {
                            view! { {contact.email} }.into_any()
                        }}
                    </td>
                    <td data-label="Phone">
                        {if contact.phone.is_empty() {
                            view! { <span class="muted">"—"</span> }.into_any()
                        } else {
                            view! { {contact.phone} }.into_any()
                        }}
                    </td>
                </tr>
            }
        })
        .collect_view()
        .into_any()
}

#[component]
fn NewContactPage() -> impl IntoView {
    let navigate = use_navigate();
    let name = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let saving = RwSignal::new(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        error.set(None);
        saving.set(true);

        let name_value = name.get_untracked();
        let phone_value = phone.get_untracked();
        let email_value = email.get_untracked();
        let navigate = navigate.clone();

        leptos::task::spawn_local(async move {
            match create_contact(name_value, phone_value, email_value).await {
                Ok(_) => navigate("/", Default::default()),
                Err(err) => {
                    error.set(Some(err.to_string()));
                    saving.set(false);
                }
            }
        });
    };

    view! {
        <section class="page">
            <div class="page-toolbar">
                <h2 class="page-heading">"New contact"</h2>
                <A href="/" attr:class="button button-secondary">"Back"</A>
            </div>

            <form class="contact-form" on:submit=on_submit>
                <div class="form-field">
                    <label for="contact-name">"Name"</label>
                    <input
                        id="contact-name"
                        type="text"
                        name="name"
                        autocomplete="name"
                        required
                        prop:value=move || name.get()
                        on:input=move |ev| name.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-field">
                    <label for="contact-email">"Email"</label>
                    <input
                        id="contact-email"
                        type="email"
                        name="email"
                        autocomplete="email"
                        prop:value=move || email.get()
                        on:input=move |ev| email.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-field">
                    <label for="contact-phone">"Phone"</label>
                    <input
                        id="contact-phone"
                        type="tel"
                        name="phone"
                        autocomplete="tel"
                        prop:value=move || phone.get()
                        on:input=move |ev| phone.set(event_target_value(&ev))
                    />
                </div>

                {move || {
                    error.get().map(|message| view! {
                        <p class="form-error" role="alert">{message}</p>
                    })
                }}

                <button
                    class="button button-primary button-full"
                    type="submit"
                    disabled=move || saving.get()
                >
                    {move || if saving.get() { "Saving..." } else { "Save contact" }}
                </button>
            </form>
        </section>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <section class="page">
            <h2 class="page-heading">"Not found"</h2>
            <p>"That page does not exist."</p>
            <A href="/" attr:class="button button-secondary">"Go home"</A>
        </section>
    }
}
