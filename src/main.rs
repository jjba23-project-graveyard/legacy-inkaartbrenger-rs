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
use ansi_term::Colour;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::writer::Writer;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::sleep;
use url::Url;

/// A simple CLI to crawl a website and generate a sitemap.xml
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The base URL to start crawling from (e.g., https://example.com)
    #[arg(short, long)]
    url: String,

    /// Number of concurrent crawler tasks
    #[arg(short, long, default_value_t = 3)]
    concurrency: usize,
}

/// Shared state for the crawler
struct CrawlerState {
    links_to_visit: Mutex<VecDeque<Url>>,
    visited_links: Mutex<HashSet<Url>>,
    sitemap_urls: Mutex<HashSet<String>>,
    base_domain: String,
}

fn log_intro() {
    println!("");
    println!("{}", Colour::Green.paint("---------------------"));
    println!("{}", Colour::Green.paint("the-sitemap-crawler"));
    println!("{}", Colour::Green.paint("---------------------"));
}

#[derive(Debug, Clone)]
struct SiteCrawlError {
    cause: String,
}

impl fmt::Display for SiteCrawlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error ocurred: {}", &self.cause)
    }
}

#[tokio::main]
async fn main() -> Result<(), SiteCrawlError> {
    log_intro();
    let args = Args::parse();
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
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ]),
    );
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

                tasks.spawn(async move {
                    crawl_url(client_clone, state_clone, url).await;
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

/// Fetches a single URL, parses it for links, and adds new, valid links to the queue
async fn crawl_url(client: Client, state: Arc<CrawlerState>, url: Url) {
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

/// Writes all discovered URLs to a sitemap.xml file
fn generate_sitemap(state: Arc<CrawlerState>) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create("sitemap.xml")?;
    let buf_writer = BufWriter::new(file);
    let mut writer = Writer::new(buf_writer);

    // Write XML declaration
    writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new(
        "1.0",
        Some("UTF-8"),
        None,
    )))?;

    // Start <urlset>
    let mut urlset = BytesStart::new("urlset");
    urlset.push_attribute(("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9"));
    writer.write_event(Event::Start(urlset))?;

    let urls = state.sitemap_urls.lock().unwrap();
    for url_str in urls.iter() {
        // Start <url>
        writer.write_event(Event::Start(BytesStart::new("url")))?;

        // <loc>...</loc>
        writer.write_event(Event::Start(BytesStart::new("loc")))?;
        writer.write_event(Event::Text(BytesText::new(url_str)))?;
        writer.write_event(Event::End(BytesEnd::new("loc")))?;

        // End <url>
        writer.write_event(Event::End(BytesEnd::new("url")))?;
    }

    // End <urlset>
    writer.write_event(Event::End(BytesEnd::new("urlset")))?;

    // Flush writer
    writer.into_inner().flush()?;

    Ok(())
}
