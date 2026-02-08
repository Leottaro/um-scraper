use std::{error::Error, hash::Hash, time::Duration};

use serde::{Deserialize, Serialize};
use thirtyfour::prelude::*;

#[derive(Debug, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct Grade {
    pub code: String,
    pub label: String,
    pub session1: String,
    pub resultat1: String,
    pub session2: String,
    pub resultat2: String,
    pub rang: String,
}

impl Grade {
    pub async fn from_row(grade_row: WebElement) -> Result<Self, Box<dyn Error>> {
        let grade_elements = grade_row
            .query(By::Css("[class=\"v-label v-widget v-has-width\"]"))
            .wait(Duration::from_secs(10), Duration::from_millis(500))
            .all_from_selector()
            .await?;

        if grade_elements.len() < 7 {
            return Err("Not enough elements found in the row".into());
        }

        let session1 = grade_elements[2].text().await?.trim().to_string();
        if session1.is_empty() {
            return Err("No grade yet.".into());
        }

        let code = grade_elements[0].text().await?.trim().to_string();
        let label = grade_elements[1].text().await?.trim().to_string();
        let resultat1 = grade_elements[3].text().await?.trim().to_string();
        let session2 = grade_elements[4].text().await?.trim().to_string();
        let resultat2 = grade_elements[5].text().await?.trim().to_string();
        let rang = grade_elements[6].text().await?.trim().to_string();

        let grade = Grade {
            code,
            label,
            session1,
            resultat1,
            session2,
            resultat2,
            rang,
        };

        log::info!("{:?}", grade);

        Ok(grade)
    }
}

impl Eq for Grade {}
