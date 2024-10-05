use std::{borrow::Borrow, sync::LazyLock};

use genanki_rs::{basic_model, Deck, Note};
use regex::Regex;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    input_dir: String,
    output_dir: String,
    deck_id: i64,
    deck_name: String,
    deck_description: String,
}

fn read_config() -> Config {
    let config_file = std::fs::read_to_string("./config.json").unwrap();
    serde_json::from_str(&config_file).unwrap()
}

fn remove_links(input: String) -> String {
    static LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(\[|\])"#).unwrap());
    let link_regex = LINK_REGEX.clone();
    link_regex.replace_all(&input, "").into_owned()
}

fn replace_math(input: String) -> String {
    static MATH_REGEX_DOUBLE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"(\${2})(?<content>.*)(\${2})"#).unwrap());
    static MATHE_REGEX_SINGLE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"(\$)(?<content>.*)(\$)"#).unwrap());
    let double = MATH_REGEX_DOUBLE.clone();
    let single = MATHE_REGEX_SINGLE.clone();
    let replaced_double = double.replace_all(&input, r#"\[$content\]"#);
    single
        .replace_all(&replaced_double, r#"\($content\)"#)
        .into_owned()
}

// Read config.
// Go through all input files. Get the title with regex. Get the rest of the content with a regex. Generate new anki note.
// Put note into deck.
// Save deck to file
fn main() {
    let config = read_config();
    let mut deck = Deck::new(config.deck_id, &config.deck_name, &config.deck_description);
    let title_regex = Regex::new(r#"#\s+(?<title>.*|\s+)\n"#).unwrap();
    for i in std::fs::read_dir(config.input_dir).unwrap() {
        let path = i.unwrap().path();
        println!(
            "Reading file: {}",
            path.clone().into_os_string().into_string().unwrap()
        );
        let file_content = std::fs::read_to_string(path).unwrap();
        let mut title = String::from("");
        let mut body = String::from("");
        for (j, line) in file_content.lines().enumerate() {
            if j == 0 {
                title = markdown::to_html(
                    title_regex
                        .captures(&format!("{}\n", line))
                        .unwrap()
                        .name("title")
                        .unwrap()
                        .as_str(),
                );
            } else {
                body = format!("{}{}\n", body, line);
            }
        }
        body = remove_links(body);
        body = replace_math(body);
        body = markdown::to_html(&body);
        deck.add_note(
            Note::new(basic_model(), vec![&title, &body]).expect("Couldn't generate note"),
        );
    }
    deck.write_to_file(&format!("{}/{}.apkg", config.output_dir, config.deck_name))
        .unwrap();
}
