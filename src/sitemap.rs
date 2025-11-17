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

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::writer::Writer;

use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::sync::Arc;

/// Writes all discovered URLs to a sitemap.xml file
pub fn generate_sitemap(state: Arc<CrawlerState>) -> Result<(), Box<dyn std::error::Error>> {
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
        // let last_modified: Result<Option<DateTime<Utc>>, String> =
        //     get_last_modified_date(url_str).await;

        // let s = format!("last modified: {}", last_modified.unwrap().unwrap());
        // println!("{}", s);
        // Start <url>
        writer.write_event(Event::Start(BytesStart::new("url")))?;

        // <loc>...</loc>
        writer.write_event(Event::Start(BytesStart::new("loc")))?;
        writer.write_event(Event::Text(BytesText::new(url_str)))?;
        writer.write_event(Event::End(BytesEnd::new("loc")))?;

        // <lastmod>...</lastmod>
        // writer.write_event(Event::Start(BytesStart::new("lastmod")))?;
        // writer.write_event(Event::Text(BytesText::new("")));
        // writer.write_event(Event::End(BytesEnd::new("lastmod")))?;

        // End <url>
        writer.write_event(Event::End(BytesEnd::new("url")))?;
    }

    // End <urlset>
    writer.write_event(Event::End(BytesEnd::new("urlset")))?;

    // Flush writer
    writer.into_inner().flush()?;

    Ok(())
}
