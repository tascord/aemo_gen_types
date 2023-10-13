use std::fs;

use aemo_gen_types::{
    get_child_by_tag, inner_text, remove_lists, setup_dom, DictionaryField, Record, RecordField,
};
use html_parser::Dom;
use regex::Regex;

static IGNORED_LINES: [&'static str; 11] = [
    "Doc Ref:",
    "1 March",
    "Page",
    "STTM REPORTS SPECIFICATIONS",
    "Field",
    "Data type",
    "Associated Reports",
    "Column Name",
    "Not Null",
    "Primary Key",
    "Comment",
];

static INITIAL_TRIGGER: &str = "For the purpose of this document,";
static FINAL_TRIGGER: &str = "The following hub references";

fn parse_dictionary_html(dom: Dom) -> Vec<String> {
    let html =
        get_child_by_tag(&dom.children, "html").expect("Misshapen HTML (Failed to find <html>)");
    let body =
        get_child_by_tag(&html.children, "body").expect("Misshapen HTML (Failed to find <body>)");

    let mut total: Vec<String> = Vec::new();
    let mut page = 0;

    body.children
        .iter()
        .filter(|e| e.element().is_some() && e.element().unwrap().id.is_some())
        .map(|e| e.element().unwrap())
        .for_each(|e| {
            page += 1;

            let lines = e
                .children
                .iter()
                .filter(|e| e.element().is_some())
                .map(|e| inner_text(e))
                .filter(|t| !t.is_empty())
                .filter(|t| !IGNORED_LINES.iter().any(|l| t.starts_with(l)))
                .collect::<Vec<String>>();

            total.extend(lines);
        });

    total = total
        .into_iter()
        .flat_map(|l| match Regex::new("[^,] {2,}").unwrap().is_match(&l) {
            true => Regex::new(" +")
                .unwrap()
                .replace(&l, " ")
                .split(" ")
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>(),
            false => vec![l],
        })
        .collect::<Vec<String>>();

    let initial_line = total
        .iter()
        .position(|l| l.starts_with(INITIAL_TRIGGER))
        .expect("Failed to find initial line");

    let final_line = total
        .iter()
        .position(|l| l.starts_with(FINAL_TRIGGER))
        .expect("Failed to find final line");

    total = total[(initial_line + 1)..final_line].to_vec();

    total
}

fn parse_data_dictionary(table: Vec<String>) -> Vec<DictionaryField> {
    let mut fields: Vec<DictionaryField> = Vec::new();

    let mut current_field = DictionaryField::default();
    let mut index = 0;
    for line in table.iter() {
        match index {
            0 => current_field.field = line.clone(),
            1 => current_field.data_type = line.clone(),
            2 => {
                current_field.reports.extend(
                    line.split(",")
                        .map(|f| f.trim().to_string())
                        .collect::<Vec<String>>(),
                );

                fields.push(current_field.clone());

                index = 0;
                current_field = DictionaryField::default();
                continue;
            }
            _ => {}
        }

        index += 1;
    }

    fields
}

pub fn get_data_dictionary(html: String) -> Vec<DictionaryField> {
    parse_data_dictionary(parse_dictionary_html(setup_dom(html)))
}

pub fn load_dictionary(path: &str) -> Vec<DictionaryField> {
    let data = fs::read_to_string(path).expect("Unable to read Data Dictionary");
    serde_json::from_str(&data).expect("Unable to parse Data Dictionary")
}

//

pub fn fetch_field(fields: Vec<DictionaryField>, field_name: &str, id: &str) -> DictionaryField {
    // println!("Fetching field {} for report {}", field_name, id);
    fields
        .iter()
        .filter(|f| f.field == field_name && f.reports.contains(&id.to_string()))
        .cloned()
        .next()
        .expect(&format!(
            "Failed to find field {field_name} for report {id}"
        ))
}

pub fn field_exists(fields: Vec<DictionaryField>, field_name: &str) -> bool {
    fields
        .iter()
        .filter(|f| f.field == field_name)
        .cloned()
        .next()
        .is_some()
}

fn parse_records_html(dom: Dom, dictionary: Vec<DictionaryField>) -> Vec<(String, Vec<String>)> {
    let html =
        get_child_by_tag(&dom.children, "html").expect("Misshapen HTML (Failed to find <html>)");
    let body =
        get_child_by_tag(&html.children, "body").expect("Misshapen HTML (Failed to find <body>)");

    let mut records: Vec<(String, Vec<String>)> = Vec::new();
    body.children
        .iter()
        .filter(|e| e.element().is_some() && e.element().unwrap().id.is_some())
        .map(|e| e.element().unwrap())
        .for_each(|e| {
            let mut lines = remove_lists(
                e.children
                    .iter()
                    .filter(|e| e.element().is_some())
                    .map(|e| inner_text(e))
                    .filter(|t| !t.is_empty())
                    .collect::<Vec<String>>(),
            )
            .into_iter()
            .flat_map(
                |t| match (t.contains("True") || t.contains("False")) && t.contains(" ") {
                    true => t
                        .split(" ")
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<String>>(),
                    false => vec![t],
                },
            )
            .filter(|t| {
                t == "True"
                    || t == "False"
                    || field_exists(dictionary.clone(), t)
                    || t.contains("~")
            })
            .collect::<Vec<String>>();

            if lines.len() > 0 {
                let record = match lines[0].contains("~") {
                    false => {
                        if records.len() == 0 {
                            println!("Started parsing mid-record, skipping page");
                            return;
                        } else {
                            records.last_mut().unwrap()
                        }
                    }
                    true => {
                        let record_line = lines[0].clone();
                        let mut record_id = record_line.split("_").next().unwrap().to_uppercase();

                        if let Some(version) =
                            Regex::new(r"_v([0-9])+_").unwrap().captures(&record_line)
                        {
                            let version = version.get(1).unwrap().as_str();
                            if version != "1" {
                                record_id += &format!("v{}", version);
                            }
                        }

                        println!("Found record name {}", record_id);
                        lines = lines[1..].to_vec();
                        records.push((record_id, Vec::new()));
                        records.last_mut().unwrap()
                    }
                };

                record.1.extend(lines);
            }
        });

    records
}

fn parse_records(
    table: Vec<(String, Vec<String>)>,
    dictionary: Vec<DictionaryField>,
) -> Vec<Record> {
    let mut records: Vec<Record> = Vec::new();
    for (id, lines) in table {
        let mut record = Record::default();
        record.name = id;

        let mut index = 0;
        let mut record_field = RecordField::default();

        for line in lines.iter() {
            match index {
                0 => record_field.name = line.clone(),
                1 => record_field.nullable = line == "False",
                3 => {}
                2 => {
                    record_field.data_type = fetch_field(
                        dictionary.clone(),
                        record_field.name.as_str(),
                        record.name.as_str(),
                    )
                    .data_type
                    .clone();

                    record.fields.push(record_field.clone());
                    record_field = RecordField::default();
                    index = 0;
                    continue;
                }
                _ => {}
            }

            index += 1;
        }
        records.push(record);
    }

    records
}

pub fn get_records(html: String, dictionary: Vec<DictionaryField>) -> Vec<Record> {
    parse_records(
        parse_records_html(setup_dom(html), dictionary.clone()),
        dictionary,
    )
}
