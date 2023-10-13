use std::{
    fs,
    process::{Command, Stdio},
};

use html_parser::{Dom, Element, Node};
use regex::Regex;
use serde::{Deserialize, Serialize};

// PDF

pub fn create_html(pdf_path: &str, pages: Option<(i32, i32)>) -> String {
    Command::new("which")
        .arg("pdftohtml")
        .stdout(Stdio::null())
        .status()
        .expect("pdftohtml not installed");

    let mut command = Command::new(r"pdftohtml");
    command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("-c")
        .arg("-i")
        .arg("-s")
        .arg("-noframes");

    match pages {
        Some((start, end)) => {
            command
                .arg("-f")
                .arg(start.to_string())
                .arg("-l")
                .arg(end.to_string());
        }
        None => {}
    }

    command.arg(pdf_path).arg("./out.html");
    command.status().expect("Unable to create HTML");
    let data = fs::read_to_string("./out.html").expect("Unable to read HTML");
    fs::remove_file("./out.html").expect("Unable to remove HTML");
    return data;
}

// HTML

pub fn setup_dom(html: String) -> Dom {
    Dom::parse(&html).expect("Unable to load HTML")
}

pub fn get_child_by_tag(children: &Vec<Node>, tag: &str) -> Option<Element> {
    match children
        .iter()
        .find(|e| match e.element() {
            Some(el) => el.name == tag,
            None => false,
        })
        .cloned()
    {
        Some(e) => e.element().cloned(),
        None => None,
    }
}

fn dig(node: &Node, text: &mut String) -> String {
    if node.text().is_some() {
        text.push_str(node.text().unwrap());
    }
    if node.element().is_some() {
        node.element().unwrap().children.iter().for_each(|e| {
            text.push_str(e.text().unwrap_or(""));
        });
    }

    return text.to_string();
}

fn clean(text: &str) -> String {
    let mut cleaned: String = text.to_string();

    // Numerical Index
    cleaned = Regex::new("&#.+?;")
        .unwrap()
        .replace_all(&cleaned, " ")
        .trim()
        .to_string();

    cleaned = Regex::new("\n")
        .unwrap()
        .replace_all(&cleaned, " ")
        .trim()
        .to_string();

    return cleaned;
}

pub fn inner_text(node: &Node) -> String {
    return clean(&dig(node, &mut String::new()));
}

// Fields

#[derive(Clone, Serialize, Deserialize)]
pub struct DictionaryField {
    pub field: String,
    pub data_type: String,
    pub reports: Vec<String>,
}

impl Default for DictionaryField {
    fn default() -> Self {
        DictionaryField {
            field: String::new(),
            data_type: String::new(),
            reports: Vec::new(),
        }
    }
}

// Records

#[derive(Clone, Serialize)]
pub struct RecordField {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

impl Default for RecordField {
    fn default() -> Self {
        RecordField {
            name: String::new(),
            data_type: String::new(),
            nullable: false,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct Record {
    pub name: String,
    pub fields: Vec<RecordField>,
}

impl Default for Record {
    fn default() -> Self {
        Record {
            name: String::new(),
            fields: Vec::new(),
        }
    }
}

// Misc
pub fn remove_lists(text: Vec<String>) -> Vec<String> {
    let mut text = text;
    let mut index = 0;
    let mut prev_line = String::new();
    for line in text.clone().iter() {
        if prev_line.contains("•") {
            text[index] = String::new();
        }
        if line.contains("•") {
            text[index] = String::new();
        }
        prev_line = line.clone();
        index += 1;
    }

    text
}
