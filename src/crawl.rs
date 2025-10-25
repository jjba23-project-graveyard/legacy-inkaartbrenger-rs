/**
  the-site-crawler

  Copyright © Josep Bigorra <jjbigorra@gmail.com>

  the-site-crawler is free software: you can redistribute it and/or modify
  it under the terms of the GNU Affero General Public License as published by
  the Free Software Foundation, either version 3 of the License, or
  (at your option) any later version.

  the-site-crawler is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU Affero General Public License for more details.

  You should have received a copy of the GNU Affero General Public License
  along with the-site-crawler.  If not, see <https://www.gnu.org/licenses/>.
*/
use crate::domain::CrawlerState;

use ansi_term::Colour;
use reqwest::Client;
use scraper::{Html, Selector};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

/// Fetches a single URL, parses it for links, and adds new, valid links to the queue
pub async fn crawl_url(client: Client, state: Arc<CrawlerState>, url: Url) {
    sleep(Duration::from_millis(400)).await;

    // Add to sitemap. We'll clean up fragments (#) later
    state.sitemap_urls.lock().unwrap().insert(url.to_string());

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

    let body = match response.text().await {
        Ok(text) => text,
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
