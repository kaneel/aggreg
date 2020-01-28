mod lib;

extern crate clap;
extern crate futures;
extern crate isahc;
extern crate scraper;
extern crate serde_json;
extern crate yaml_rust;

use clap::{App, Arg};
use futures::{executor::block_on, future::join_all};
use std::{
    fs::{create_dir_all, read_dir, write},
    io::{Error, ErrorKind},
    path::Path,
};
use yaml_rust::Yaml;

use lib::config::Config;
use lib::entry::Entry;
use lib::feed::Feed;

use isahc::HttpClient;

fn main() {
    match block_on(async_main()) {
        Err(error) => println!("{:?}", error.to_string()),
        _ => {}
    }
}

async fn async_main() -> Result<(), Error> {
    let matches = App::new("aggreg")
        .version("1.0")
        .about("Does awesome things")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("The path to the config file (defaults to ./.aggreg)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("The output path (defaults to ./out)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("limit")
                .short("l")
                .long("limit")
                .help("The maximum number of articles to output")
                .takes_value(true),
        )
        .get_matches();

    let config: Result<Config, Error> =
        Config::from(matches.value_of("config").unwrap_or(".aggreg"));

    match config {
        Ok(Config { contents, .. }) => {
            let output = match &contents["output"] {
                Yaml::String(str) => str.to_string(),
                _ => matches.value_of("output").unwrap_or("./out").to_string(),
            };

            let limit = match &contents["limit"] {
                Yaml::Integer(int) => *int,
                _ => matches
                    .value_of("limit")
                    .unwrap_or("5")
                    .parse::<i64>()
                    .unwrap(),
            } as usize;

            let feeds = aggreg(contents, limit).await?;

            read_dir_and_create(output.clone())?;

            let result = feeds
                .into_iter()
                .map(|feed| {
                    let filename = format!("{}/{}.json", output, feed.key);
                    write_file(filename, feed.to_json().to_string())
                })
                .collect::<Vec<Result<(), Error>>>();

            for item in result {
                match item {
                    Err(error) => {
                        println!("{}", error.to_string());
                    }
                    _ => {}
                }
            }

            Ok(())
        }
        Err(error) => {
            match error.kind() {
                ErrorKind::NotFound => {
                    println!("Config file not found, either use -c path/to/config or create a .aggreg file");
                    Err(error)
                }
                ErrorKind::Other => {
                    println!("{:?}", error.to_string());
                    Err(error)
                }
                _ => Err(error),
            }
        }
    }
}

async fn aggreg(contents: Yaml, limit: usize) -> Result<Vec<Feed>, Error> {
    let feeds = contents["feeds"].as_vec();
    let client = HttpClient::new()?;

    match feeds {
        Some(f) => {
            let result = f.iter().map(|feed| {
                let hash_list = feed.as_hash().unwrap();
                let key = hash_list.front().unwrap().0;
                let options = hash_list.front().unwrap().1;
                get_feed(key.as_str().unwrap().to_string(), options, limit, &client)
            });

            let all_feeds = join_all(result).await;

            Ok(all_feeds)
        }
        None => Err(Error::new(ErrorKind::Other, "no feeds")),
    }
}

async fn get_feed(key: String, options: &Yaml, limit: usize, client: &HttpClient) -> Feed {
    let entry = Entry::from(&key, options);
    let request = client.get_async(&entry.url).await;

    let feed = match request {
        Ok(response) => Feed::from(
            response.into_body().text_async().await.unwrap(),
            limit,
            entry,
        ),
        Err(error) => Feed::err(entry, error),
    };

    feed
}

fn read_dir_and_create(path: String) -> Result<(), Error> {
    let path = Path::new(path.as_str());

    match read_dir(path) {
        Err(_) => {
            println!("output directory {:?} will be created", path);
            create_dir_all(path)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn write_file(filename: String, json: String) -> Result<(), Error> {
    let path = Path::new(filename.as_str());
    let display = path.display();

    match write(path, json.as_bytes()) {
        Err(why) => Err(Error::new(
            ErrorKind::Other,
            format!("couldn't write to {}: {}", display, why.to_string()),
        )),
        _ => Ok(()),
    }
}
