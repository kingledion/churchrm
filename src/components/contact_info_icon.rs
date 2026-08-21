use leptos::prelude::*;

use crate::models::ContactCard;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContactInfoStatus {
    Both,
    Partial,
    None,
}

impl ContactInfoStatus {
    pub fn from_emails_phones(emails: &[String], phones: &[String]) -> Self {
        match (!emails.is_empty(), !phones.is_empty()) {
            (true, true) => Self::Both,
            (true, false) | (false, true) => Self::Partial,
            (false, false) => Self::None,
        }
    }

    pub fn from_card(card: &ContactCard) -> Self {
        Self::from_emails_phones(&card.emails, &card.phones)
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
pub fn ContactInfoIcon(status: ContactInfoStatus) -> impl IntoView {
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
