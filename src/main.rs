use clap::Parser;
use feedbin_api::models::{CacheRequestResponse, Entry};
use feedbin_api::FeedbinApi;
use lazy_static::lazy_static;
use url::Url;

lazy_static! {
    static ref FEEDBIN_BASE_URL: Url =
        Url::parse("https://api.feedbin.com/").expect("Could not parse Feedbin API Base URL");
}

/// Cleaning tool for your Feedbin account
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Your Feedbin username
    #[arg(short, long)]
    username: String,

    /// Your Feedbin password
    #[arg(short, long)]
    password: String,

    /// Names of tags to clean up feeds from
    #[arg(short, long, required = true)]
    tagged: Vec<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let client = reqwest::Client::new();
    eprintln!("Created HTTP client");

    let api = FeedbinApi::new(&FEEDBIN_BASE_URL, args.username, args.password);
    eprintln!("Created API client");

    let mut target_feed_ids: Vec<u64> = Vec::new();

    {
        let taggings_response = api.get_taggings(&client, None).await.unwrap();

        eprintln!("Taggings requested");

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

    for feed_id in target_feed_ids.iter() {
        eprintln!(" - {}", feed_id);
    }

    eprintln!("Getting unread entry IDs");
    let all_unread_entry_ids = api.get_unread_entry_ids(&client).await.unwrap();
    eprintln!("There are {} unread entries", all_unread_entry_ids.len());

    eprintln!("Getting entries for above feed IDs");

    let mut tagged_unread_entries: Vec<Entry> = Vec::new();

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

    eprintln!("Tagged Entries: {:#?}", tagged_unread_entries);

    // There is a limit of 1,000 entry_ids per request
    // api.set_entries_read(&client, matching_entries).await.unwrap();
}
