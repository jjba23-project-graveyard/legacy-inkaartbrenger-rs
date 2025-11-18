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
use ansi_term::Colour;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// A simple CLI to crawl a website and generate a sitemap.xml
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The base URL to start crawling from (e.g., https://example.com)
    #[arg(short, long)]
    pub url: String,

    /// Number of concurrent crawler tasks
    #[arg(short, long, default_value_t = 3)]
    pub concurrency: usize,
}

pub fn parsed_cli_args() -> Args {
    Args::parse()
}

pub fn log_intro() {
    println!("");
    println!("{}", Colour::Green.bold().paint("\ninkaartbrenger\n"));
}

pub fn prepare_progress_bar() -> ProgressBar {
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
    pb
}
