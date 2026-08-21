use std::time::Duration;

use leptos::prelude::*;
use uuid::Uuid;

use crate::components::LifeStagePicker;
use crate::models::{LifeStage, Person};
use crate::server::update_person;

#[derive(Clone, PartialEq, Eq)]
pub struct PersonDraft {
    pub name: String,
    pub phone: String,
    pub email: String,
    pub life_stage: Option<LifeStage>,
}

impl PersonDraft {
    pub fn from_person(person: &Person) -> Self {
        Self {
            name: person.name.clone(),
            phone: person.phone.clone(),
            email: person.email.clone(),
            life_stage: person.life_stage,
        }
    }

    pub fn empty() -> Self {
        Self {
            name: String::new(),
            phone: String::new(),
            email: String::new(),
            life_stage: None,
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

#[component]
pub fn PersonEditor(
    /// Stable id used for input element ids.
    #[prop(into)]
    editor_id: String,
    name: RwSignal<String>,
    phone: RwSignal<String>,
    email: RwSignal<String>,
    life_stage: RwSignal<Option<LifeStage>>,
    #[prop(optional)] show_heading_name: bool,
) -> impl IntoView {
    let name_input_id = format!("{editor_id}-name");
    let email_input_id = format!("{editor_id}-email");
    let phone_input_id = format!("{editor_id}-phone");
    let stage_label_id = format!("{editor_id}-life-stage");

    view! {
        <div class="person-editor">
            {if show_heading_name {
                view! {
                    <label class="sr-only" for=name_input_id.clone()>"Name"</label>
                    <input
                        id=name_input_id.clone()
                        class="contact-detail__name-input person-editor__name-input"
                        type="text"
                        name="name"
                        autocomplete="name"
                        placeholder="Name"
                        prop:value=move || name.get()
                        on:input=move |ev| name.set(event_target_value(&ev))
                    />
                }
                .into_any()
            } else {
                view! {
                    <div class="form-field">
                        <label for=name_input_id.clone()>"Name"</label>
                        <input
                            id=name_input_id.clone()
                            type="text"
                            name="name"
                            autocomplete="name"
                            prop:value=move || name.get()
                            on:input=move |ev| name.set(event_target_value(&ev))
                        />
                    </div>
                }
                .into_any()
            }}

            <div class="form-field">
                <span class="form-field__label" id=stage_label_id.clone()>"Life stage"</span>
                <LifeStagePicker selected=life_stage labelled_by=stage_label_id.clone()/>
                <p class="form-hint">
                    {move || match life_stage.get() {
                        Some(stage) => stage.description(),
                        None => "Optional. Tap an icon to set, tap again to clear.",
                    }}
                </p>
            </div>

            {move || {
                if life_stage.get().is_some_and(LifeStage::is_child) {
                    view! { <></> }.into_any()
                } else {
                    view! {
                        <div class="form-field">
                            <label for=email_input_id.clone()>"Email"</label>
                            <input
                                id=email_input_id.clone()
                                type="email"
                                name="email"
                                autocomplete="email"
                                prop:value=move || email.get()
                                on:input=move |ev| email.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="form-field">
                            <label for=phone_input_id.clone()>"Phone"</label>
                            <input
                                id=phone_input_id.clone()
                                type="tel"
                                name="phone"
                                autocomplete="tel"
                                prop:value=move || phone.get()
                                on:input=move |ev| phone.set(event_target_value(&ev))
                            />
                        </div>
                    }
                    .into_any()
                }
            }}
        </div>
    }
}

/// Debounced autosave for a persisted person. Returns status/error signals for the parent.
pub fn use_person_autosave(
    person_id: Uuid,
    name: RwSignal<String>,
    phone: RwSignal<String>,
    email: RwSignal<String>,
    life_stage: RwSignal<Option<LifeStage>>,
    last_saved: RwSignal<PersonDraft>,
) -> (RwSignal<Option<String>>, RwSignal<Option<String>>, RwSignal<u64>) {
    let save_status = RwSignal::new(None::<String>);
    let save_error = RwSignal::new(None::<String>);
    let save_generation = RwSignal::new(0u64);
    let initialized = StoredValue::new(false);

    Effect::new(move |_| {
        let draft = PersonDraft {
            name: name.get(),
            phone: phone.get(),
            email: email.get(),
            life_stage: life_stage.get(),
        };

        if !initialized.get_value() {
            initialized.set_value(true);
            return;
        }

        if draft == last_saved.get_untracked() {
            return;
        }

        save_generation.update(|generation| *generation += 1);
        let generation = save_generation.get_untracked();
        save_status.set(Some("Saving...".to_string()));
        save_error.set(None);

        set_timeout(
            move || {
                if save_generation.get_untracked() != generation {
                    return;
                }

                leptos::task::spawn_local(async move {
                    if save_generation.get_untracked() != generation {
                        return;
                    }

                    match update_person(
                        person_id,
                        draft.name.clone(),
                        draft.phone.clone(),
                        draft.email.clone(),
                        draft.life_stage,
                    )
                    .await
                    {
                        Ok(saved) => {
                            if save_generation.get_untracked() != generation {
                                return;
                            }
                            last_saved.set(PersonDraft::from_person(&saved));
                            save_status.set(Some("Saved".to_string()));
                            save_error.set(None);
                        }
                        Err(err) => {
                            if save_generation.get_untracked() != generation {
                                return;
                            }
                            save_status.set(None);
                            save_error.set(Some(err.to_string()));
                        }
                    }
                });
            },
            Duration::from_millis(400),
        );
    });

    (save_status, save_error, save_generation)
}
