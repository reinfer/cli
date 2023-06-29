use anyhow::{anyhow, Result};
use dialoguer::Confirm;
use url::Url;

pub mod config;
pub mod create;
pub mod delete;
pub mod get;
pub mod update;

pub fn ensure_uip_user_consents_to_ai_unit_charge(base_url: &Url) -> Result<()> {
    if base_url
        .origin()
        .ascii_serialization()
        .to_lowercase()
        .ends_with("reinfer.io")
    {
        return Ok(());
    }

    if Confirm::new()
        .with_prompt(
            r#"🚨⚠️ 👉 CAUTION 👈⚠️ 🚨

The operation you are about to perform will charge AI units.

Do you want to continue?"#,
        )
        .interact()?
    {
        Ok(())
    } else {
        Err(anyhow!("Billable operation aborted by user"))
    }
}
