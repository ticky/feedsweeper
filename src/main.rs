use clap::Parser;
use feedbin_api::models::CacheRequestResponse;
use feedbin_api::FeedbinApi;
use lazy_static::lazy_static;
use std::collections::HashMap;
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
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let client = reqwest::Client::new();
    eprintln!("Created HTTP client");

    let api = FeedbinApi::new(&FEEDBIN_BASE_URL, args.username, args.password);
    eprintln!("Created API client");

    let mut tag_map: HashMap<String, Vec<u64>> = HashMap::new();

    {
        let taggings_response = api.get_taggings(&client, None).await.unwrap();
        eprintln!("Taggings requested");

        if let CacheRequestResponse::Modified(taggings) = taggings_response {
            for tagging in &taggings.value {
                if tag_map.contains_key(&tagging.name) {
                    let feed_list = tag_map
                        .get_mut(&tagging.name)
                        .expect("Could not get tagging which should have existed");
                    feed_list.push(tagging.feed_id);
                } else {
                    tag_map.insert(tagging.name.clone(), vec![tagging.feed_id]);
                }
            }
        }
    };

    for (key, val) in tag_map.iter() {
        println!("Tagging \"{}\" contains {} feed(s):", key, val.len());

        for feed_id in val.iter() {
            println!(" - {}", feed_id);
        }
    }
}
