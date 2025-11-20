/**
  inkaartbrenger

  Copyright © Josep Bigorra <jjbigorra@gmail.com>

  inkaartbrenger is free software: you can redistribute it and/or modify
  it under the terms of the GNU Affero General Public License as published by
  the Free Software Foundation, either version 3 of the License, or
  (at your option) any later version.

  inkaartbrenger is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU Affero General Public License for more details.

  You should have received a copy of the GNU Affero General Public License
  along with inkaartbrenger.  If not, see <https://www.gnu.org/licenses/>.
*/
use crate::domain::CrawlerState;

use ansi_term::Colour;
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use reqwest::header::LAST_MODIFIED;
use reqwest::{Client, Response};
use scraper::{Html, Selector};
use std::sync::Arc;

use url::Url;

pub async fn crawl_url(client: Client, state: Arc<CrawlerState>, url: Url) -> Result<()> {
    let response: Response = client.get(url.clone()).send().await.inspect_err(|e| {
        eprintln!(
            "\n{} Failed to fetch {}: {}",
            Colour::Red.paint("Error:"),
            url,
            e
        )
    })?;

    if response.status().is_success() {
        Ok(())
    } else {
        eprintln!(
            "\n{} Got status {} for {}",
            Colour::Red.paint("Error:"),
            response.status(),
            url
        );
        Err(anyhow!("Response status was not successful"))
    }?;

    let maybe_last_modified: Option<String> = get_last_modified_date_from_response(&response)
        .inspect_err(|e| {
            eprintln!(
                "\n{} Failed to read last modified header from {}: {}",
                Colour::Red.paint("Error:"),
                url,
                e
            )
        })?;

    // --- Start of critical section ---
    state.sitemap_urls.lock().unwrap().insert(url.to_string());
    maybe_last_modified.inspect(|last_modified| {
        state
            .last_modified
            .lock()
            .unwrap()
            .insert(url.to_string(), last_modified.to_string());
    });
    // --- End of critical section ---

    let document: Html = response
        .text()
        .await
        .map(|r| r.to_string())
        .map(|s| Html::parse_document(&s))?;

    discover_push_links(state, url, document)?;

    Ok(())
}

fn discover_push_links(state: Arc<CrawlerState>, url: Url, document: Html) -> Result<()> {
    let link_selector = Selector::parse("a[href]").map_err(|e| anyhow!(e.to_string()))?;

    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            match url.join(href) {
                Ok(mut new_url) => {
                    // Remove fragments (e.g., #section)
                    new_url.set_fragment(None);

                    if new_url.domain() == Some(&state.base_domain) {
                        // --- Start of critical section ---
                        let visited = state.visited_links.lock().unwrap();
                        let mut queue = state.links_to_visit.lock().unwrap();

                        if !visited.contains(&new_url) && !queue.contains(&new_url) {
                            queue.push_back(new_url);
                        }
                        // --- End of critical section ---
                    }
                }
                Err(_) => {
                    // Invalid link, e.g., "mailto:foo@bar.com" or "javascript:..."
                }
            }
        }
    }

    Ok(())
}

fn get_last_modified_date_from_response(response: &Response) -> Result<Option<String>> {
    let Some(header_value) = response.headers().get(LAST_MODIFIED) else {
        return Ok(None);
    };
    let header_str = header_value.to_str().unwrap_or_default();
    let datetime: DateTime<chrono::FixedOffset> = DateTime::parse_from_rfc2822(header_str)
        .context("Failed to parse the Last-Modified header to a RFC2822 compliant date-time")?;

    Ok(Some(
        datetime.with_timezone(&Utc).format("%Y-%m-%d").to_string(),
    ))
}
