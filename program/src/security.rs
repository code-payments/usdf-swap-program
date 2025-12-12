#[cfg(not(feature = "no-entrypoint"))]
use {solana_security_txt::security_txt};

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "USDF Swap Program",
    project_url: "https://flipcash.com",
    contacts: "email:security@flipcash.com",
    policy: "https://github.com/code-payments/usdf-swap-program/blob/main/SECURITY.md",
    source_code: "https://github.com/code-payments/usdf-swap-program",
    auditors: "todo"
}
