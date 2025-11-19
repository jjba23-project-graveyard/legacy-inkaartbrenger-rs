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
pub mod cli;
pub mod crawl;
pub mod domain;
pub mod sitemap;

use crate::cli::{log_intro, parsed_cli_args, prepare_progress_bar};
use crate::domain::{CrawlerState, SiteCrawlError};
use crate::sitemap::generate_sitemap;
use ansi_term::Colour;
use rand::prelude::*;

use crawl::crawl_url;

use tokio::time::sleep;

use reqwest::Client;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinSet;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), SiteCrawlError> {
    log_intro();
    let args = parsed_cli_args();
    let base_url = Url::parse(&args.url).map_err(|e| SiteCrawlError {
        cause: e.to_string(),
    })?;
    let domain_name = base_url
        .domain()
        .ok_or(SiteCrawlError {
            cause: "Invalid domain name".to_string(),
        })?
        .to_owned();

    // Initialize shared state
    let mut initial_links = VecDeque::new();
    initial_links.push_back(base_url.clone());

    let state = Arc::new(CrawlerState {
        links_to_visit: Mutex::new(initial_links),
        visited_links: Mutex::new(HashSet::new()),
        sitemap_urls: Mutex::new(HashSet::new()),
        last_modified: Mutex::new(HashMap::new()),
        base_domain: domain_name,
    });

    // Setup HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("SimpleRustSitemapCrawler/0.1.0")
        .build()
        .map_err(|e| SiteCrawlError {
            cause: e.to_string(),
        })?;

    // Setup progress bar
    let pb = prepare_progress_bar();
    pb.set_message("Crawling...");

    let mut tasks = JoinSet::new();
    let mut total_crawled = 0;

    loop {
        let mut url_to_crawl = None;

        // --- Start of critical section ---
        // Lock the queue to get a new URL
        {
            let mut queue = state.links_to_visit.lock().unwrap();
            if let Some(url) = queue.pop_front() {
                // Check if we've already visited this URL
                let mut visited = state.visited_links.lock().unwrap();
                if visited.insert(url.clone()) {
                    // If it's a new URL, we'll crawl it
                    url_to_crawl = Some(url);
                }
            }
        }
        // --- End of critical section ---

        if let Some(url) = url_to_crawl {
            // We have a new URL to crawl, spawn a task if we have capacity
            if tasks.len() < args.concurrency {
                total_crawled += 1;
                pb.set_message(format!(
                    "Crawled: {} | Pending: {} | Crawling: {}",
                    total_crawled,
                    state.links_to_visit.lock().unwrap().len(),
                    url
                ));

                let state_clone = Arc::clone(&state);
                let client_clone = client.clone();

                let mut rng = rand::rng();
                let mut nums: Vec<u64> = (1..100).collect();
                nums.shuffle(&mut rng);
                let random_jitter_millis = nums.choose(&mut rng).unwrap_or(&50);
                let task_initial_delay = Duration::from_millis(250 + random_jitter_millis);

                tasks.spawn(async move {
                    sleep(task_initial_delay).await;
                    if let Err(_) = tokio::time::timeout(
                        Duration::from_millis(7200),
                        crawl_url(client_clone, state_clone, url.clone()),
                    )
                    .await
                    {
                        eprintln!(
                            "\n{} Timeout while crawling URL: {}",
                            Colour::Red.paint("Error:"),
                            url.clone()
                        );
                    }
                });
            } else {
                // We're at capacity, wait for a task to finish
                if tasks.join_next().await.is_none()
                    && state.links_to_visit.lock().unwrap().is_empty()
                {
                    // No tasks running and queue is empty
                    break;
                }
                // Re-add the URL to the front of the queue to try again after a task finishes
                state.links_to_visit.lock().unwrap().push_front(url);
            }
        } else if tasks.is_empty() {
            // Queue is empty and no tasks are running, we're done!
            break;
        } else {
            // Queue is empty, but tasks are still running. Wait for them.
            tasks.join_next().await;
        }
    }

    pb.finish_with_message(format!(
        "{} Crawled {} pages.",
        Colour::Green.bold().paint("✔"),
        total_crawled
    ));

    // --- Generate Sitemap ---
    println!(
        "{} Generating sitemap.xml...",
        Colour::Cyan.bold().paint("->")
    );
    generate_sitemap(state).map_err(|e| SiteCrawlError {
        cause: e.to_string(),
    })?;

    println!(
        "\n{} {}",
        Colour::Green.bold().paint("✔ Success!"),
        "sitemap.xml generated."
    );

    Ok(())
}
