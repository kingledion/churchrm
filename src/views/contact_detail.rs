use leptos::prelude::*;
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_params_map},
    NavigateOptions,
};
use uuid::Uuid;

use crate::components::{
    use_person_autosave, KindToggle, PersonDraft, PersonEditor,
};
use crate::models::{ContactDetail, ContactKind, Person};
use crate::server::{
    add_family_member, convert_contact_to_family, convert_contact_to_person, create_contact,
    delete_contact, delete_family_member, get_contact_detail, update_person,
};

fn confirm_delete() -> bool {
    #[cfg(feature = "hydrate")]
    {
        web_sys::window()
            .and_then(|window| {
                window
                    .confirm_with_message(
                        "Delete this contact? This cannot be undone from the app.",
                    )
                    .ok()
            })
            .unwrap_or(false)
    }
    #[cfg(not(feature = "hydrate"))]
    {
        true
    }
}

#[component]
pub fn NewContactPage() -> impl IntoView {
    view! {
        <section class="page">
            <ContactComposer initial=None/>
        </section>
    }
}

#[component]
pub fn ContactDetailPage() -> impl IntoView {
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
                Some(id) => get_contact_detail(id).await.map(Some),
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
                        Some(Ok(Some(detail))) => Some(
                            view! { <ContactComposer initial=Some(detail)/> }.into_any(),
                        ),
                        Some(Ok(None)) | Some(Err(_)) => Some(
                            view! {
                                <div class="page-toolbar">
                                    <h2 class="page-heading">"Contact not found"</h2>
                                    <A href="/" attr:class="button button-secondary">"Back"</A>
                                </div>
                                <p class="empty-state">"That contact does not exist."</p>
                            }
                            .into_any(),
                        ),
                    }
                }}
            </Transition>
        </section>
    }
}

#[component]
fn ContactComposer(initial: Option<ContactDetail>) -> impl IntoView {
    let navigate = use_navigate();
    let navigate_back = navigate.clone();
    let navigate_after_delete = navigate.clone();
    let detail = RwSignal::new(initial.clone());
    let kind = RwSignal::new(
        initial
            .as_ref()
            .map(ContactDetail::kind)
            .unwrap_or(ContactKind::Person),
    );
    let save_status = RwSignal::new(None::<String>);
    let save_error = RwSignal::new(None::<String>);
    let converting = RwSignal::new(false);
    let create_started = StoredValue::new(false);
    let local_person = LocalPersonState::new(
        initial
            .as_ref()
            .and_then(|detail| match detail {
                ContactDetail::Person { person, .. } => Some(person.clone()),
                ContactDetail::Family { members, .. } => members.first().cloned(),
            })
            .unwrap_or_else(|| Person::draft(None)),
    );
    let local_members = RwSignal::new(match &initial {
        Some(ContactDetail::Family { members, .. }) => {
            members.iter().cloned().map(MemberState::from_person).collect::<Vec<_>>()
        }
        _ => vec![MemberState::from_person(Person::draft(None))],
    });

    // Keep kind in sync when detail is replaced after convert/create.
    Effect::new(move |_| {
        if let Some(current) = detail.get() {
            kind.set(current.kind());
        }
    });

    // Kind toggle: local-only before persist; convert APIs after.
    Effect::new(move |_| {
        let selected = kind.get();
        let Some(current) = detail.get() else {
            return;
        };
        if selected == current.kind() || converting.get_untracked() {
            return;
        }

        // Persisted family contacts cannot convert back to person.
        if selected == ContactKind::Person
            && matches!(current, ContactDetail::Family { .. })
        {
            kind.set(ContactKind::Family);
            return;
        }

        converting.set(true);
        save_error.set(None);
        save_status.set(Some("Updating...".to_string()));
        let contact_id = current.contact_id();
        let previous_kind = current.kind();
        let local_person = local_person.clone();

        leptos::task::spawn_local(async move {
            let result = match selected {
                ContactKind::Family => convert_contact_to_family(contact_id).await,
                ContactKind::Person => convert_contact_to_person(contact_id).await,
            };

            match result {
                Ok(updated) => {
                    sync_local_from_detail(&updated, &local_person, &local_members);
                    detail.set(Some(updated));
                    save_status.set(Some("Saved".to_string()));
                    save_error.set(None);
                }
                Err(err) => {
                    kind.set(previous_kind);
                    save_status.set(None);
                    save_error.set(Some(err.to_string()));
                }
            }
            converting.set(false);
        });
    });

    // Local draft: first non-empty change creates the contact then replaces the URL.
    Effect::new(move |_| {
        if detail.get().is_some() {
            return;
        }

        let selected = kind.get();
        let person_draft = PersonDraft {
            name: local_person.name.get(),
            phone: local_person.phone.get(),
            email: local_person.email.get(),
            life_stage: local_person.life_stage.get(),
        };
        let member_drafts: Vec<PersonDraft> = local_members
            .get()
            .into_iter()
            .map(|member| PersonDraft {
                name: member.name.get(),
                phone: member.phone.get(),
                email: member.email.get(),
                life_stage: member.life_stage.get(),
            })
            .collect();

        let dirty = match selected {
            ContactKind::Person => !person_draft.is_empty(),
            ContactKind::Family => member_drafts.iter().any(|draft| !draft.is_empty()),
        };
        if !dirty || converting.get_untracked() || create_started.get_value() {
            return;
        }

        create_started.set_value(true);
        converting.set(true);
        save_status.set(Some("Saving...".to_string()));
        save_error.set(None);

        let navigate = navigate.clone();

        leptos::task::spawn_local(async move {
            match create_contact(selected).await {
                Ok(created) => {
                    let persist_result = match &created {
                        ContactDetail::Person { person, .. } => {
                            update_person(
                                person.id,
                                person_draft.name,
                                person_draft.phone,
                                person_draft.email,
                                person_draft.life_stage,
                            )
                            .await
                            .map(|_| ())
                        }
                        ContactDetail::Family {
                            members,
                            family_id,
                            ..
                        } => {
                            let mut result = Ok(());
                            for (index, member) in members.iter().enumerate() {
                                let draft = member_drafts
                                    .get(index)
                                    .cloned()
                                    .unwrap_or_else(PersonDraft::empty);
                                if let Err(err) = update_person(
                                    member.id,
                                    draft.name,
                                    draft.phone,
                                    draft.email,
                                    draft.life_stage,
                                )
                                .await
                                {
                                    result = Err(err);
                                    break;
                                }
                            }
                            if result.is_ok() {
                                for draft in member_drafts.iter().skip(members.len()) {
                                    match add_family_member(*family_id).await {
                                        Ok(person) => {
                                            if let Err(err) = update_person(
                                                person.id,
                                                draft.name.clone(),
                                                draft.phone.clone(),
                                                draft.email.clone(),
                                                draft.life_stage,
                                            )
                                            .await
                                            {
                                                result = Err(err);
                                                break;
                                            }
                                        }
                                        Err(err) => {
                                            result = Err(err);
                                            break;
                                        }
                                    }
                                }
                            }
                            result
                        }
                    };

                    match persist_result {
                        Ok(()) => match get_contact_detail(created.contact_id()).await {
                            Ok(fresh) => {
                                navigate(
                                    &format!("/contacts/{}", fresh.contact_id()),
                                    NavigateOptions {
                                        replace: true,
                                        ..Default::default()
                                    },
                                );
                            }
                            Err(err) => {
                                save_status.set(None);
                                save_error.set(Some(err.to_string()));
                                create_started.set_value(false);
                                converting.set(false);
                            }
                        },
                        Err(err) => {
                            save_status.set(None);
                            save_error.set(Some(err.to_string()));
                            create_started.set_value(false);
                            converting.set(false);
                        }
                    }
                }
                Err(err) => {
                    save_status.set(None);
                    save_error.set(Some(err.to_string()));
                    create_started.set_value(false);
                    converting.set(false);
                }
            }
        });
    });

    on_cleanup(move || {
        let Some(current) = detail.get_untracked() else {
            return;
        };
        leptos::task::spawn_local(async move {
            match get_contact_detail(current.contact_id()).await {
                Ok(fresh) if fresh.is_empty() => {
                    let _ = delete_contact(fresh.contact_id()).await;
                }
                _ => {}
            }
        });
    });

    let on_back = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        navigate_back("/", Default::default());
    };

    let deleting = RwSignal::new(false);
    let can_delete = Signal::derive(move || detail.get().is_some());
    // Family contacts cannot convert back to person in the UI.
    let allow_person = Signal::derive(move || {
        !matches!(detail.get(), Some(ContactDetail::Family { .. }))
    });
    let allow_family = Signal::derive(move || true);

    view! {
        <div class="contact-detail__top">
            <div class="contact-detail__heading">
                <a
                    class="back-caret"
                    href="/"
                    aria-label="Back to contacts"
                    style="text-decoration: none"
                    on:click=on_back
                >
                    "<"
                </a>
                <span class="contact-detail__heading-label">
                    {move || match kind.get() {
                        ContactKind::Person => "Person",
                        ContactKind::Family => "Family",
                    }}
                </span>
            </div>
            <Show when=move || can_delete.get()>
                <button
                    type="button"
                    class="button button-danger"
                    disabled=move || deleting.get()
                    on:click={
                        let navigate = navigate_after_delete.clone();
                        move |_| {
                            let Some(current) = detail.get_untracked() else {
                                return;
                            };
                            if deleting.get_untracked() {
                                return;
                            }
                            if !confirm_delete() {
                                return;
                            }

                            deleting.set(true);
                            save_error.set(None);
                            let navigate = navigate.clone();
                            leptos::task::spawn_local(async move {
                                match delete_contact(current.contact_id()).await {
                                    Ok(()) => navigate("/", Default::default()),
                                    Err(err) => {
                                        save_error.set(Some(err.to_string()));
                                        deleting.set(false);
                                    }
                                }
                            });
                        }
                    }
                >
                    {move || {
                        if deleting.get() {
                            "Deleting..."
                        } else {
                            "Delete contact"
                        }
                    }}
                </button>
            </Show>
        </div>

        <KindToggle kind=kind allow_person=allow_person allow_family=allow_family/>

        <div class="contact-form contact-detail__form">
            {move || match (detail.get(), kind.get()) {
                (Some(ContactDetail::Person { .. }), _) | (None, ContactKind::Person) => {
                    let person_id = detail.get().and_then(|current| match current {
                        ContactDetail::Person { person, .. } => Some(person.id),
                        _ => None,
                    });
                    view! {
                        <PersistedOrLocalPerson
                            person_id=person_id
                            state=local_person.clone()
                            save_status=save_status
                            save_error=save_error
                        />
                    }
                    .into_any()
                }
                (Some(ContactDetail::Family { .. }), _) | (None, ContactKind::Family) => {
                    let family_id = detail.get().and_then(|current| match current {
                        ContactDetail::Family { family_id, .. } => Some(family_id),
                        _ => None,
                    });
                    view! {
                        <FamilyEditor
                            family_id=family_id
                            members=local_members
                            detail=detail
                            save_status=save_status
                            save_error=save_error
                        />
                    }
                    .into_any()
                }
            }}

            <p
                class=move || {
                    if save_error.get().is_some() {
                        "autosave-status form-error"
                    } else {
                        "autosave-status muted"
                    }
                }
                aria-live="polite"
            >
                {move || {
                    save_error
                        .get()
                        .or_else(|| save_status.get())
                        .unwrap_or_default()
                }}
            </p>
        </div>

        <section class="contact-notes">
            <h3 class="contact-notes__heading">"Notes"</h3>
            <p class="muted">"Notes will appear here in a future update."</p>
        </section>
    }
}

#[derive(Clone, Copy)]
struct LocalPersonState {
    name: RwSignal<String>,
    phone: RwSignal<String>,
    email: RwSignal<String>,
    life_stage: RwSignal<Option<crate::models::LifeStage>>,
    last_saved: RwSignal<PersonDraft>,
}

impl LocalPersonState {
    fn new(person: Person) -> Self {
        let draft = PersonDraft::from_person(&person);
        Self {
            name: RwSignal::new(draft.name.clone()),
            phone: RwSignal::new(draft.phone.clone()),
            email: RwSignal::new(draft.email.clone()),
            life_stage: RwSignal::new(draft.life_stage),
            last_saved: RwSignal::new(draft),
        }
    }

    fn set_from_person(&self, person: &Person) {
        let draft = PersonDraft::from_person(person);
        self.name.set(draft.name.clone());
        self.phone.set(draft.phone.clone());
        self.email.set(draft.email.clone());
        self.life_stage.set(draft.life_stage);
        self.last_saved.set(draft);
    }
}

#[derive(Clone)]
struct MemberState {
    id: Option<Uuid>,
    name: RwSignal<String>,
    phone: RwSignal<String>,
    email: RwSignal<String>,
    life_stage: RwSignal<Option<crate::models::LifeStage>>,
    last_saved: RwSignal<PersonDraft>,
}

impl MemberState {
    fn from_person(person: Person) -> Self {
        let draft = PersonDraft::from_person(&person);
        Self {
            id: Some(person.id),
            name: RwSignal::new(draft.name.clone()),
            phone: RwSignal::new(draft.phone.clone()),
            email: RwSignal::new(draft.email.clone()),
            life_stage: RwSignal::new(draft.life_stage),
            last_saved: RwSignal::new(draft),
        }
    }

    fn local_empty() -> Self {
        let draft = PersonDraft::empty();
        Self {
            id: None,
            name: RwSignal::new(String::new()),
            phone: RwSignal::new(String::new()),
            email: RwSignal::new(String::new()),
            life_stage: RwSignal::new(None),
            last_saved: RwSignal::new(draft),
        }
    }
}

fn sync_local_from_detail(
    detail: &ContactDetail,
    local_person: &LocalPersonState,
    local_members: &RwSignal<Vec<MemberState>>,
) {
    match detail {
        ContactDetail::Person { person, .. } => {
            local_person.set_from_person(person);
            local_members.set(vec![MemberState::from_person(person.clone())]);
        }
        ContactDetail::Family { members, .. } => {
            if let Some(first) = members.first() {
                local_person.set_from_person(first);
            }
            local_members.set(
                members
                    .iter()
                    .cloned()
                    .map(MemberState::from_person)
                    .collect(),
            );
        }
    }
}

#[component]
fn PersistedOrLocalPerson(
    person_id: Option<Uuid>,
    state: LocalPersonState,
    save_status: RwSignal<Option<String>>,
    save_error: RwSignal<Option<String>>,
) -> impl IntoView {
    if let Some(person_id) = person_id {
        let (status, error, _) = use_person_autosave(
            person_id,
            state.name,
            state.phone,
            state.email,
            state.life_stage,
            state.last_saved,
        );
        Effect::new(move |_| {
            save_status.set(status.get());
        });
        Effect::new(move |_| {
            save_error.set(error.get());
        });
    }

    view! {
        <div class="contact-detail__heading contact-detail__heading--nested">
            <PersonEditor
                editor_id="person"
                name=state.name
                phone=state.phone
                email=state.email
                life_stage=state.life_stage
                show_heading_name=true
            />
        </div>
    }
}

#[component]
fn FamilyEditor(
    family_id: Option<Uuid>,
    members: RwSignal<Vec<MemberState>>,
    detail: RwSignal<Option<ContactDetail>>,
    save_status: RwSignal<Option<String>>,
    save_error: RwSignal<Option<String>>,
) -> impl IntoView {
    let adding = RwSignal::new(false);

    let on_add = move |_| {
        let Some(family_id) = family_id else {
            members.update(|list| list.push(MemberState::local_empty()));
            return;
        };
        if adding.get_untracked() {
            return;
        }
        adding.set(true);
        leptos::task::spawn_local(async move {
            match add_family_member(family_id).await {
                Ok(person) => {
                    members.update(|list| list.push(MemberState::from_person(person)));
                    if let Some(current) = detail.get_untracked() {
                        if let Ok(fresh) = get_contact_detail(current.contact_id()).await {
                            detail.set(Some(fresh));
                        }
                    }
                    save_status.set(Some("Saved".to_string()));
                }
                Err(err) => save_error.set(Some(err.to_string())),
            }
            adding.set(false);
        });
    };

    view! {
        <div class="family-editor">
            {move || {
                members
                    .get()
                    .into_iter()
                    .enumerate()
                    .map(|(index, member)| {
                        view! {
                            <FamilyMemberBlock
                                index=index
                                member=member
                                family_id=family_id
                                members=members
                                detail=detail
                                save_status=save_status
                                save_error=save_error
                            />
                        }
                    })
                    .collect_view()
            }}
            <button
                type="button"
                class="button button-secondary"
                disabled=move || adding.get()
                on:click=on_add
            >
                {move || if adding.get() { "Adding..." } else { "Add person" }}
            </button>
        </div>
    }
}

#[component]
fn FamilyMemberBlock(
    index: usize,
    member: MemberState,
    family_id: Option<Uuid>,
    members: RwSignal<Vec<MemberState>>,
    detail: RwSignal<Option<ContactDetail>>,
    save_status: RwSignal<Option<String>>,
    save_error: RwSignal<Option<String>>,
) -> impl IntoView {
    if let Some(person_id) = member.id {
        let (status, error, _) = use_person_autosave(
            person_id,
            member.name,
            member.phone,
            member.email,
            member.life_stage,
            member.last_saved,
        );
        Effect::new(move |_| {
            save_status.set(status.get());
        });
        Effect::new(move |_| {
            save_error.set(error.get());
        });
    }

    let can_remove = move || members.get().len() > 1;
    let on_remove = move |_| {
        if !can_remove() {
            return;
        }
        if let (Some(_family_id), Some(person_id)) = (family_id, member.id) {
            leptos::task::spawn_local(async move {
                match delete_family_member(person_id).await {
                    Ok(()) => {
                        members.update(|list| {
                            list.retain(|item| item.id != Some(person_id));
                        });
                        if let Some(current) = detail.get_untracked() {
                            if let Ok(fresh) = get_contact_detail(current.contact_id()).await {
                                detail.set(Some(fresh));
                            }
                        }
                        save_status.set(Some("Saved".to_string()));
                    }
                    Err(err) => save_error.set(Some(err.to_string())),
                }
            });
        } else {
            members.update(|list| {
                if list.len() > 1 {
                    list.remove(index);
                }
            });
        }
    };

    view! {
        <div class="family-member">
            <div class="family-member__header">
                <h3 class="family-member__title">
                    {format!("Person {}", index + 1)}
                </h3>
                <button
                    type="button"
                    class="button button-secondary family-member__remove"
                    disabled=move || !can_remove()
                    on:click=on_remove
                >
                    "Remove"
                </button>
            </div>
            <PersonEditor
                editor_id=format!("member-{index}")
                name=member.name
                phone=member.phone
                email=member.email
                life_stage=member.life_stage
                show_heading_name=false
            />
        </div>
    }
}
