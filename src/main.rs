use clap::Parser;
use feedbin_api::models::{CacheRequestResponse, Entry as FeedbinEntry};
use feedbin_api::FeedbinApi;
use lazy_static::lazy_static;

lazy_static! {
    static ref FEEDBIN_BASE_URL: url::Url =
        url::Url::parse("https://api.feedbin.com/").expect("Could not parse Feedbin API Base URL");
}

/// Cleaning tool for your Feedbin account
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Your Feedbin username
    #[arg(short, long, env = "FEEDBIN_USERNAME")]
    username: String,

    /// Your Feedbin password
    #[arg(short, long, env = "FEEDBIN_PASSWORD")]
    password: String,

    /// Names of tags to clean up feeds from
    #[arg(short, long, required = true)]
    tagged: Vec<String>,

    /// Maximum entry age to keep unread.
    ///
    /// Specify a duration such as "1 week", "3M" (3 months), "10d" (10 days). You can specify multiple time spans if you need more specificity, such as "1d 12h".
    ///
    /// Supported time span suffixes include: min, minute, minutes, m, hr, hour, hours, h, day, days, d, week, weeks, w, month, months, M, year, years, y.
    #[arg(short, long)]
    max_age: Option<humantime::Duration>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // eprintln!("{:#?}", args);

    let client = reqwest::Client::new();

    let api = FeedbinApi::new(&FEEDBIN_BASE_URL, args.username, args.password);

    let mut target_feed_ids: Vec<u64> = Vec::new();

    {
        let taggings_response = api.get_taggings(&client, None).await.unwrap();

        if let CacheRequestResponse::Modified(taggings) = taggings_response {
            // TODO: Error if no matches found
            for tagging in &taggings.value {
                // TODO: Should this be a glob match, maybe?
                if args.tagged.contains(&tagging.name) {
                    target_feed_ids.push(tagging.feed_id);
                }
            }
        }
    };

    // Sort the IDs so that we can `binary_search` it later
    target_feed_ids.sort();

    eprintln!("Found these matching feed IDs:");
    for feed_id in target_feed_ids.iter() {
        eprintln!(" - {}", feed_id);
    }

    let all_unread_entry_ids = api.get_unread_entry_ids(&client).await.unwrap();
    eprintln!("Found {} unread entries", all_unread_entry_ids.len());

    let mut tagged_unread_entries: Vec<FeedbinEntry> = Vec::new();

    // We can only request 100 entries per page, so we chunk the above list
    for (index, unread_entry_page) in all_unread_entry_ids.chunks(100).enumerate() {
        let mut entries_response = api
            .get_entries(
                &client,
                None,
                None,
                Some(unread_entry_page),
                None,
                None,
                false,
            )
            .await
            .unwrap();

        let total_page_entries = entries_response.len();

        entries_response.retain(|entry| target_feed_ids.binary_search(&entry.feed_id).is_ok());

        eprintln!(
            "Tagged entries in page {}: {}/{}",
            index,
            entries_response.len(),
            total_page_entries
        );

        tagged_unread_entries.append(&mut entries_response);
    }

    let mut to_mark_unread_entry_ids: Vec<u64> = Vec::new();

    if let Some(max_age) = args.max_age {
        let mut aged_out_entry_ids: Vec<u64> = Vec::new();

        let reference_time = chrono::Local::now();
        let oldest_allowed = reference_time - *max_age;

        for unread_entry in tagged_unread_entries.into_iter() {
            // TODO: Don't panic here
            let entry_published_date =
                chrono::DateTime::parse_from_rfc3339(&unread_entry.published).unwrap();

            if entry_published_date < oldest_allowed {
                eprintln!("Matching entry: {:#?}", unread_entry);
                aged_out_entry_ids.push(unread_entry.id)
            }
        }

        println!(
            "Max age: {} entries to mark as unread due to being older than {}",
            aged_out_entry_ids.len(),
            oldest_allowed
        );

        to_mark_unread_entry_ids.append(&mut aged_out_entry_ids);
    }

    if to_mark_unread_entry_ids.len() == 0 {
        println!("No entries to mark as read!");
    } else {
        // We can only mark 1,000 entries as read per request, so we chunk the above list
        for (index, matching_entry_page) in to_mark_unread_entry_ids.chunks(1_000).enumerate() {
            eprintln!(
                "Marking {} on page {} as read",
                matching_entry_page.len(),
                index
            );
            api.set_entries_read(&client, matching_entry_page)
                .await
                .unwrap();
        }
    }

    println!("Done!");
}
