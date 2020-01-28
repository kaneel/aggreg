use scraper::Selector;
use yaml_rust::Yaml;

pub struct Entry {
    pub key: String,
    pub url: String,
    pub name: String,
    anchor: String,
    content: String,
    selector: String,
    title: String,
}

impl Entry {
    fn get_from_yaml(option: &Yaml) -> String {
        let result = option.as_str();

        match result {
            Some(st) => st.to_string(),
            None => String::from(""),
        }
    }

    pub fn from(key: &String, options: &Yaml) -> Entry {
        let url = Entry::get_from_yaml(&options["url"]);
        let title = Entry::get_from_yaml(&options["title"]);
        let selector = Entry::get_from_yaml(&options["selector"]);
        let anchor = Entry::get_from_yaml(&options["anchor"]);
        let content = Entry::get_from_yaml(&options["content"]);
        let name = Entry::get_from_yaml(&options["name"]);

        Entry {
            key: key.as_str().to_string(),
            name,
            url,
            title,
            selector,
            anchor,
            content,
        }
    }

    pub fn to_selectors(&self) -> Vec<(&str, Selector, Option<&str>)> {
        let items: Vec<(&str, String, Option<&str>)> = vec![
            ("selector", self.selector.clone(), None),
            ("title", self.title.clone(), None),
            ("anchor", self.anchor.clone(), Some("href")),
            ("content", self.content.clone(), None),
        ];

        items
            .iter()
            .filter(|(_, value, ..)| value != "")
            .map(|(key, value, attribute)| (*key, Selector::parse(value).unwrap(), *attribute))
            .collect()
    }
}
