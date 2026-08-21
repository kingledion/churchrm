use leptos::prelude::*;

use crate::models::LifeStage;

fn life_stage_class(life_stage: Option<LifeStage>) -> &'static str {
    match life_stage {
        Some(LifeStage::Child) => "life-stage-icon life-stage-icon--child",
        Some(LifeStage::YoungAdult) => "life-stage-icon life-stage-icon--young-adult",
        Some(LifeStage::Parent) => "life-stage-icon life-stage-icon--parent",
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

fn life_stage_glyph(life_stage: Option<LifeStage>) -> impl IntoView {
    match life_stage {
        Some(LifeStage::Child) => view! {
            <circle cx="12" cy="9" r="2.4" fill="currentColor"/>
            <path
                fill="currentColor"
                d="M8.2 20.5v-.7c0-1.8 1.7-3.3 3.8-3.3s3.8 1.5 3.8 3.3v.7H8.2z"
            />
        }
        .into_any(),
        Some(LifeStage::YoungAdult) => view! {
            <circle cx="12" cy="6.5" r="3" fill="currentColor"/>
            <path
                fill="currentColor"
                d="M7 21v-1.25C7 17.24 9.24 15 12 15s5 2.24 5 4.75V21H7z"
            />
        }
        .into_any(),
        Some(LifeStage::Older) => view! {
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
        Some(LifeStage::Parent) => view! {
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
        None => view! {
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
pub fn LifeStageIcon(life_stage: Option<LifeStage>) -> impl IntoView {
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
pub fn LifeStagePicker(
    selected: RwSignal<Option<LifeStage>>,
    #[prop(optional, into)] labelled_by: Option<String>,
) -> impl IntoView {
    let labelled_by = labelled_by.unwrap_or_else(|| "life-stage-label".to_string());

    view! {
        <div
            class="life-stage-picker"
            role="radiogroup"
            aria-labelledby=labelled_by
        >
            {LifeStage::ALL
                .into_iter()
                .map(|stage| {
                    let option_class = move || {
                        let base = match stage {
                            LifeStage::Child => "life-stage-option life-stage-option--child",
                            LifeStage::YoungAdult => {
                                "life-stage-option life-stage-option--young-adult"
                            }
                            LifeStage::Parent => "life-stage-option life-stage-option--parent",
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

fn kind_person_glyph() -> impl IntoView {
    view! {
        <svg
            class="kind-toggle__glyph"
            viewBox="0 0 24 24"
            width="1.25rem"
            height="1.25rem"
            aria-hidden="true"
            focusable="false"
        >
            <circle cx="12" cy="6.5" r="3" fill="currentColor"/>
            <path
                fill="currentColor"
                d="M7 21v-1.25C7 17.24 9.24 15 12 15s5 2.24 5 4.75V21H7z"
            />
        </svg>
    }
}

fn kind_family_glyph() -> impl IntoView {
    view! {
        <svg
            class="kind-toggle__glyph"
            viewBox="0 0 24 24"
            width="1.25rem"
            height="1.25rem"
            aria-hidden="true"
            focusable="false"
        >
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
        </svg>
    }
}

#[component]
pub fn KindToggle(
    kind: RwSignal<crate::models::ContactKind>,
    /// When false, the Person control is a greyed non-interactive icon (family contacts).
    allow_person: Signal<bool>,
    /// When false, the Family control is a greyed non-interactive icon.
    allow_family: Signal<bool>,
) -> impl IntoView {
    use crate::models::ContactKind;

    view! {
        <div class="kind-toggle" role="radiogroup" aria-label="Contact type">
            {move || {
                let selected = kind.get() == ContactKind::Person;
                let allowed = allow_person.get();
                kind_toggle_option(
                    "Person",
                    selected,
                    allowed,
                    kind_person_glyph().into_any(),
                    move || kind.set(ContactKind::Person),
                )
            }}
            {move || {
                let selected = kind.get() == ContactKind::Family;
                let allowed = allow_family.get();
                kind_toggle_option(
                    "Family",
                    selected,
                    allowed,
                    kind_family_glyph().into_any(),
                    move || kind.set(ContactKind::Family),
                )
            }}
        </div>
    }
}

fn kind_toggle_option(
    label: &'static str,
    selected: bool,
    allowed: bool,
    glyph: AnyView,
    on_select: impl Fn() + 'static,
) -> AnyView {
    if selected {
        view! {
            <span
                class="kind-toggle__option kind-toggle__option--selected"
                role="radio"
                aria-checked="true"
                aria-label=label
                title=label
            >
                {glyph}
                <span class="kind-toggle__label">{label}</span>
            </span>
        }
        .into_any()
    } else if allowed {
        view! {
            <button
                type="button"
                class="kind-toggle__option kind-toggle__option--action"
                role="radio"
                aria-checked="false"
                aria-label=label
                title=label
                on:click=move |_| on_select()
            >
                {glyph}
                <span class="kind-toggle__label">{label}</span>
            </button>
        }
        .into_any()
    } else {
        view! {
            <span
                class="kind-toggle__option kind-toggle__option--disabled"
                role="radio"
                aria-checked="false"
                aria-disabled="true"
                aria-label=format!("{label} (unavailable)")
                title=format!("{label} is unavailable for this contact")
            >
                {glyph}
                <span class="kind-toggle__label">{label}</span>
            </span>
        }
        .into_any()
    }
}
