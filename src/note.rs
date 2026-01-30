use std::{error::Error, hash::Hash, time::Duration};

use serde::{Deserialize, Serialize};
use thirtyfour::prelude::*;

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub code: String,
    pub label: String,
    pub session1: String,
    pub resultat1: String,
    pub session2: String,
    pub resultat2: String,
    pub rang: String,
}

impl Note {
    pub async fn from_row(note_row: WebElement) -> Result<Self, Box<dyn Error>> {
        let note_elements = note_row
            .query(By::Css("[class=\"v-label v-widget v-has-width\"]"))
            .wait(Duration::from_secs(10), Duration::from_millis(500))
            .all_from_selector()
            .await?;

        if note_elements.len() < 7 {
            return Err("Not enough elements found in the row".into());
        }

        let session1 = note_elements[2].text().await?.trim().to_string();
        if session1.is_empty() {
            return Err("No note yet.".into());
        }

        let code = note_elements[0].text().await?.trim().to_string();
        let label = note_elements[1].text().await?.trim().to_string();
        let resultat1 = note_elements[3].text().await?.trim().to_string();
        let session2 = note_elements[4].text().await?.trim().to_string();
        let resultat2 = note_elements[5].text().await?.trim().to_string();
        let rang = note_elements[6].text().await?.trim().to_string();

        let note = Note {
            code,
            label,
            session1,
            resultat1,
            session2,
            resultat2,
            rang,
        };

        log::info!("{:?}", note);

        Ok(note)
    }
}

impl Eq for Note {}
