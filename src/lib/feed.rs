use super::entry::Entry;
use scraper::Html;
use serde::Serialize;
use serde_json::json;

pub struct Row(String, String);

#[derive(Clone, Debug, Serialize)]
pub struct FeedItem {
    pub anchor: Option<String>,
    pub content: Option<String>,
    pub title: Option<String>,
}

impl FeedItem {
    fn get_entry(entries: &Vec<Row>, name: &str) -> Option<String> {
        let entry = entries.iter().find(|Row(key, ..)| key.as_str() == name);

        match entry {
            Some(e) => Some(e.1.as_str().to_string()),
            None => None,
        }
    }
    pub fn from(entries: Vec<Row>) -> FeedItem {
        let anchor = FeedItem::get_entry(&entries, "anchor");
        let content = FeedItem::get_entry(&entries, "content");
        let title = FeedItem::get_entry(&entries, "title");

        FeedItem {
            anchor,
            content,
            title,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Feed {
    pub key: String,
    error: Option<String>,
    name: String,
    contents: Vec<FeedItem>,
}

impl Feed {
    pub fn empty_row(key: String) -> Row {
        Row(key, String::from(""))
    }

    pub fn err(entry: Entry, error: isahc::Error) -> Feed {
        let e = error.to_string();
        Feed {
            key: entry.key,
            name: entry.name,
            error: Some(e),
            contents: vec![],
        }
    }

    pub fn from(body: String, limit: usize, entry: Entry) -> Feed {
        let mut selectors = entry.to_selectors();
        let document = Html::parse_document(&body);
        let main_selector = selectors.remove(0);
        let mut elements: Vec<_> = document.select(&main_selector.1).collect();

        elements.truncate(limit);

        let feed: Vec<FeedItem> = elements
            .iter()
            .map(|element| {
                let result: Vec<Row> = selectors
                    .iter()
                    .map(|(key, selector, attribute)| {
                        let mut fields: Vec<_> = element.select(&selector).collect();
                        let new_key = key.to_string();

                        match &fields.pop() {
                            Some(field) => match attribute {
                                Some(attr) => {
                                    let value = field.value().attr(attr);
                                    match value {
                                        Some(v) => Row(new_key, v.to_string()),
                                        None => Feed::empty_row(new_key),
                                    }
                                }
                                None => {
                                    let value = field.text().collect::<Vec<_>>().join("");
                                    Row(new_key, value)
                                }
                            },
                            None => Feed::empty_row(new_key),
                        }
                    })
                    .collect();

                FeedItem::from(result)
            })
            .collect();

        Feed {
            key: entry.key,
            name: entry.name,
            error: None,
            contents: feed,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        let contents: Vec<serde_json::Value> =
            self.contents.iter().map(|item| json!(item)).collect();

        json!({
          "key": self.key,
          "name": self.name,
          "error": self.error,
          "content": contents
        })
    }
}
