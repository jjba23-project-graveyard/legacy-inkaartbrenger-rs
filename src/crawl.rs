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
use chrono::{DateTime, ParseError, Utc};
use reqwest::header::LAST_MODIFIED;
use reqwest::{Client, Response};
use scraper::{Html, Selector};
use std::sync::Arc;

use url::Url;

/// Fetches a single URL, parses it for links, and adds new, valid links to the queue
pub async fn crawl_url(client: Client, state: Arc<CrawlerState>, url: Url) {
    let response = match client.get(url.clone()).send().await {
        Ok(res) => res,
        Err(e) => {
            eprintln!(
                "\n{} Failed to fetch {}: {}",
                Colour::Red.paint("Error:"),
                url,
                e
            );
            return;
        }
    };

    if !response.status().is_success() {
        eprintln!(
            "\n{} Got status {} for {}",
            Colour::Yellow.paint("Warn:"),
            response.status(),
            url
        );
        return;
    }

    // Add to sitemap. We'll clean up fragments (#) later
    state.sitemap_urls.lock().unwrap().insert(url.to_string());

    let maybe_last_modified: Option<String> = get_last_modified_date_from_response(&response)
        .inspect_err(|e| {
            eprintln!(
                "\n{} Failed to read last modified header from {}: {}",
                Colour::Red.paint("Error:"),
                url,
                e
            )
        })
        .unwrap_or(None);

    // --- Start of critical section ---
    maybe_last_modified.inspect(|last_modified| {
        state
            .last_modified
            .lock()
            .unwrap()
            .insert(url.to_string(), last_modified.to_string());
    });
    // --- End of critical section ---

    let body: String = match response.text().await {
        Ok(text) => text.to_string(),
        Err(e) => {
            eprintln!(
                "\n{} Failed to read body from {}: {}",
                Colour::Red.paint("Error:"),
                url,
                e
            );
            return;
        }
    };

    let document = Html::parse_document(&body);
    let link_selector = Selector::parse("a[href]").unwrap();

    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            // Try to parse the link relative to the current page's URL
            match url.join(href) {
                Ok(mut new_url) => {
                    // Remove fragments (e.g., #section)
                    new_url.set_fragment(None);

                    // Check if it's on the same domain
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
    // Optional: Add logging here if you want to see links found per page
    // if new_links_found > 0 {
    //    println!("\nFound {} new links on {}", new_links_found, url);
    // }
}

fn get_last_modified_date_from_response(response: &Response) -> Result<Option<String>, ParseError> {
    let Some(header_value) = response.headers().get(LAST_MODIFIED) else {
        return Ok(None);
    };
    let header_str = header_value.to_str().unwrap_or_default();
    let datetime = DateTime::parse_from_rfc2822(header_str)?;

    Ok(Some(
        datetime.with_timezone(&Utc).format("%Y-%m-%d").to_string(),
    ))
}
