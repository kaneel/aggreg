# A Ggreg

A simple Rust project that will aggregate news from websites; taking selectors from a YAML file and exporting a JSON per website.

## CLI

* -c, --config: The path to the config file (defaults to ./.aggreg)
* -l, --limit: The maximum number of articles to fetch
* -o, --output: The output path (defaults to ./out)

## Config file

The config file also takes limit and output as well as a list of feeds formatted like in the aggreg.example.yml file in this repository. The YAML file takes precedence over CLI options.

```yml
limit: 10
output: ./some/path
feeds: 
  - kotaku:
      url: "https://www.kotaku.co.uk/"
      selector: "#maincontentlist section:last-child article.media"
      anchor: ".media__body a"
      title: ".media__body h2 a"
      content: ".media__body p"
```
