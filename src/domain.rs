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
use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::sync::Mutex;
use url::Url;

#[derive(Debug, Clone)]
pub struct SiteCrawlError {
    pub cause: String,
}

impl fmt::Display for SiteCrawlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error ocurred: {}", &self.cause)
    }
}

/// Shared state for the crawler
pub struct CrawlerState {
    pub links_to_visit: Mutex<VecDeque<Url>>,
    pub visited_links: Mutex<HashSet<Url>>,
    pub sitemap_urls: Mutex<HashSet<String>>,
    pub base_domain: String,
}
