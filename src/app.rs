use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{A, Route, Router, Routes},
    hooks::{use_navigate, use_params_map},
    ParamSegment, StaticSegment,
};
use uuid::Uuid;

use crate::models::{Contact, LifeStage};
use crate::server::{create_contact, get_contact, list_contacts};

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
                        <Route
                            path=(StaticSegment("contacts"), ParamSegment("id"))
                            view=ContactDetailPage
                        />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ContactInfoStatus {
    Both,
    Partial,
    None,
}

impl ContactInfoStatus {
    fn from_contact(contact: &Contact) -> Self {
        match (!contact.email.is_empty(), !contact.phone.is_empty()) {
            (true, true) => Self::Both,
            (true, false) | (false, true) => Self::Partial,
            (false, false) => Self::None,
        }
    }

    fn class(self) -> &'static str {
        match self {
            Self::Both => "contact-status contact-status--complete",
            Self::Partial => "contact-status contact-status--partial",
            Self::None => "contact-status contact-status--empty",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Both => "Has email and phone",
            Self::Partial => "Has partial contact info",
            Self::None => "No contact info",
        }
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let name_query = RwSignal::new(String::new());

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
                <A href="/new" attr:class="button button-primary">"New contact"</A>
            </div>

            <Transition fallback=|| view! { <p class="empty-state">"Loading contacts..."</p> }>
                {move || {
                    contacts.get().map(|rows| view! { <ContactCardList contacts=rows/> })
                }}
            </Transition>

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
fn ContactCardList(contacts: Vec<Contact>) -> impl IntoView {
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
                .map(|contact| {
                    let href = format!("/contacts/{}", contact.id);
                    let status = ContactInfoStatus::from_contact(&contact);
                    let life_stage = contact.life_stage;
                    view! {
                        <div class="contact-card">
                            <A href=href attr:class="contact-card__link">
                                <span class="contact-card__name">{contact.name}</span>
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

#[component]
fn ContactInfoIcon(status: ContactInfoStatus) -> impl IntoView {
    view! {
        <svg
            class=status.class()
            viewBox="0 0 24 24"
            width="1em"
            height="1em"
            aria-label=status.label()
            role="img"
            focusable="false"
        >
            <title>{status.label()}</title>
            <path
                fill="currentColor"
                d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4-8 5-8-5V6l8 5 8-5v2z"
            />
        </svg>
    }
}

fn life_stage_class(life_stage: Option<LifeStage>) -> &'static str {
    match life_stage {
        Some(LifeStage::Young) => "life-stage-icon life-stage-icon--young",
        Some(LifeStage::HasKids) => "life-stage-icon life-stage-icon--has-kids",
        Some(LifeStage::Older) => "life-stage-icon life-stage-icon--older",
        None => "life-stage-icon life-stage-icon--empty",
    }
}

fn life_stage_label(life_stage: Option<LifeStage>) -> &'static str {
    match life_stage {
        Some(stage) => stage.label(),
        None => "Life stage not set",
    }
}

/// Custom glyphs: young = one person, with children = adult + child, older = person with cane.
fn life_stage_glyph(life_stage: Option<LifeStage>) -> impl IntoView {
    match life_stage {
        Some(LifeStage::Young) => view! {
            // Single upright figure
            <circle cx="12" cy="6.5" r="3" fill="currentColor"/>
            <path
                fill="currentColor"
                d="M7 21v-1.25C7 17.24 9.24 15 12 15s5 2.24 5 4.75V21H7z"
            />
        }
        .into_any(),
        Some(LifeStage::HasKids) => view! {
            // Adult (left) + child (right)
            <circle cx="8.5" cy="5.5" r="2.6" fill="currentColor"/>
            <path
                fill="currentColor"
                d="M4.2 20.5v-1c0-2.2 1.9-4 4.3-4s4.3 1.8 4.3 4v1H4.2z"
            />
            <circle cx="16.5" cy="9.2" r="2" fill="currentColor"/>
            <path
                fill="currentColor"
                d="M13.4 20.5v-.7c0-1.6 1.4-2.9 3.1-2.9s3.1 1.3 3.1 2.9v.7h-6.2z"
            />
        }
        .into_any(),
        Some(LifeStage::Older) => view! {
            // Figure with walking cane
            <circle cx="10.5" cy="5.5" r="2.7" fill="currentColor"/>
            <path
                fill="currentColor"
                d="M6.2 20.5v-1.1c0-2.2 1.9-4 4.3-4 1.4 0 2.6.6 3.4 1.5l1.1-1.1 1.1 1.1-2.2 2.2v1.4H6.2z"
            />
            <path
                fill="currentColor"
                d="M15.2 11.2c.4-.3.9-.2 1.2.2l2.8 4.2c.2.3.3.6.3 1v4h-1.6v-3.7l-2.4-3.6c-.3-.4-.2-.9.2-1.2.1 0 .2-.1.3-.1z"
            />
        }
        .into_any(),
        None => view! {
            // Neutral silhouette outline when unset
            <circle
                cx="12"
                cy="7"
                r="3"
                fill="none"
                stroke="currentColor"
                stroke-width="1.75"
            />
            <path
                fill="none"
                stroke="currentColor"
                stroke-width="1.75"
                stroke-linecap="round"
                d="M6.5 20.25v-.6c0-2.7 2.5-4.9 5.5-4.9s5.5 2.2 5.5 4.9v.6"
            />
        }
        .into_any(),
    }
}

#[component]
fn LifeStageIcon(life_stage: Option<LifeStage>) -> impl IntoView {
    let label = life_stage_label(life_stage);
    let class = life_stage_class(life_stage);

    view! {
        <svg
            class=class
            viewBox="0 0 24 24"
            width="1em"
            height="1em"
            aria-label=label
            role="img"
            focusable="false"
        >
            <title>{label}</title>
            {life_stage_glyph(life_stage)}
        </svg>
    }
}

#[component]
fn LifeStagePicker(selected: RwSignal<Option<LifeStage>>) -> impl IntoView {
    view! {
        <div
            class="life-stage-picker"
            role="radiogroup"
            aria-labelledby="life-stage-label"
        >
            {LifeStage::ALL
                .into_iter()
                .map(|stage| {
                    let option_class = move || {
                        let base = match stage {
                            LifeStage::Young => "life-stage-option life-stage-option--young",
                            LifeStage::HasKids => "life-stage-option life-stage-option--has-kids",
                            LifeStage::Older => "life-stage-option life-stage-option--older",
                        };
                        if selected.get() == Some(stage) {
                            format!("{base} life-stage-option--selected")
                        } else {
                            base.to_string()
                        }
                    };

                    view! {
                        <button
                            type="button"
                            class=option_class
                            role="radio"
                            aria-checked=move || (selected.get() == Some(stage)).to_string()
                            aria-label=format!("{}: {}", stage.label(), stage.description())
                            title=stage.description()
                            on:click=move |_| {
                                selected.update(|current| {
                                    *current = if *current == Some(stage) {
                                        None
                                    } else {
                                        Some(stage)
                                    };
                                });
                            }
                        >
                            <svg
                                class="life-stage-option__icon"
                                viewBox="0 0 24 24"
                                width="1.75rem"
                                height="1.75rem"
                                aria-hidden="true"
                                focusable="false"
                            >
                                {life_stage_glyph(Some(stage))}
                            </svg>
                            <span class="life-stage-option__label">{stage.label()}</span>
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
fn ContactDetailPage() -> impl IntoView {
    let params = use_params_map();
    let contact = Resource::new(
        move || {
            params
                .read()
                .get("id")
                .and_then(|id| Uuid::parse_str(&id).ok())
        },
        |id| async move {
            match id {
                Some(id) => get_contact(id).await.map(Some),
                None => Ok(None),
            }
        },
    );

    view! {
        <section class="page">
            <Transition fallback=|| view! { <p class="empty-state">"Loading contact..."</p> }>
                {move || {
                    match contact.get() {
                        None => None,
                        Some(Ok(Some(contact))) => Some(view! {
                            <ContactDetail contact=contact/>
                        }.into_any()),
                        Some(Ok(None)) | Some(Err(_)) => Some(view! {
                            <div class="page-toolbar">
                                <h2 class="page-heading">"Contact not found"</h2>
                                <A href="/" attr:class="button button-secondary">"Back"</A>
                            </div>
                            <p class="empty-state">"That contact does not exist."</p>
                        }.into_any()),
                    }
                }}
            </Transition>
        </section>
    }
}

#[component]
fn ContactDetail(contact: Contact) -> impl IntoView {
    let email = contact.email.clone();
    let phone = contact.phone.clone();
    let life_stage = contact.life_stage;

    view! {
        <h2 class="page-heading contact-detail__heading">
            <a
                class="back-caret"
                href="/"
                aria-label="Back to contacts"
                style="text-decoration: none"
            >
                "<"
            </a>
            {contact.name}
        </h2>

        <dl class="contact-detail">
            <div class="contact-detail__row">
                <dt>"Email"</dt>
                <dd>
                    {if email.is_empty() {
                        view! { <span class="muted">"Not provided"</span> }.into_any()
                    } else {
                        view! {
                            <a class="contact-detail__link" href=format!("mailto:{email}")>
                                {email.clone()}
                            </a>
                        }
                        .into_any()
                    }}
                </dd>
            </div>
            <div class="contact-detail__row">
                <dt>"Phone"</dt>
                <dd>
                    {if phone.is_empty() {
                        view! { <span class="muted">"Not provided"</span> }.into_any()
                    } else {
                        view! {
                            <a class="contact-detail__link" href=format!("tel:{phone}")>
                                {phone.clone()}
                            </a>
                        }
                        .into_any()
                    }}
                </dd>
            </div>
            <div class="contact-detail__row">
                <dt>"Life stage"</dt>
                <dd class="contact-detail__life-stage">
                    <LifeStageIcon life_stage=life_stage/>
                    {match life_stage {
                        Some(stage) => view! { <span>{stage.label()}</span> }.into_any(),
                        None => view! { <span class="muted">"Not provided"</span> }.into_any(),
                    }}
                </dd>
            </div>
        </dl>

        <section class="contact-notes">
            <h3 class="contact-notes__heading">"Notes"</h3>
            <p class="muted">"Notes will appear here in a future update."</p>
        </section>
    }
}

#[component]
fn NewContactPage() -> impl IntoView {
    let navigate = use_navigate();
    let name = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let life_stage = RwSignal::new(None::<LifeStage>);
    let error = RwSignal::new(None::<String>);
    let saving = RwSignal::new(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        error.set(None);
        saving.set(true);

        let name_value = name.get_untracked();
        let phone_value = phone.get_untracked();
        let email_value = email.get_untracked();
        let life_stage_value = life_stage.get_untracked();
        let navigate = navigate.clone();

        leptos::task::spawn_local(async move {
            match create_contact(name_value, phone_value, email_value, life_stage_value).await {
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

                <div class="form-field">
                    <span class="form-field__label" id="life-stage-label">"Life stage"</span>
                    <LifeStagePicker selected=life_stage/>
                    <p class="form-hint">
                        {move || match life_stage.get() {
                            Some(stage) => stage.description(),
                            None => "Optional. Tap an icon to set, tap again to clear.",
                        }}
                    </p>
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
