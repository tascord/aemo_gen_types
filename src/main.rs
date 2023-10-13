use std::sync::Mutex;

use aemo_automate::{create_html, DictionaryField};
use parsers::sttm;

pub mod parsers;

fn main() {
    // Pages 17 through 26 for Data Dictionary
    // Pages 42 through 155 for Reports
    let pages = Some((42, 155));
    let html = create_html("./files/sttm.pdf", pages)
        .replace("market_postition", "market_position") // Aemo my beloved
        .replace("Facility_", "facility_");

    // Already generated Data Dictionary
    let dictionary = patch_sttm_dict(sttm::load_dictionary("./files/dictionary.json"));
    println!("Loaded Data Dictionary");

    std::fs::write(
        "./files/ir_patched_dict",
        serde_json::to_string_pretty(&dictionary).unwrap(),
    )
    .unwrap();

    // Generate records
    let records = sttm::get_records(html, dictionary);
    println!("Generated records");

    std::fs::write(
        "./files/records.json",
        serde_json::to_string_pretty(&records).expect("Unable to serialize records"),
    )
    .expect("Unable to write records to disk");

    println!("Wrote records to disk");
}

// Inject fixes for Aemo my beloved
fn patch_sttm_dict(dictionary: Vec<DictionaryField>) -> Vec<DictionaryField> {
    let dictionary = Mutex::new(dictionary);

    let report_uses = |field: &str, report: &str| {
        println!("Patching report {report} to use field {field}");
        dictionary
            .lock()
            .unwrap()
            .iter_mut()
            .find(|f| f.field == field)
            .unwrap()
            .reports
            .push(report.to_string());
    };

    let new_field = |name: &str, data_type: &str, reports: Vec<&str>| {
        println!("Patching new field {name}");
        dictionary.lock().unwrap().push(DictionaryField {
            field: name.to_string(),
            data_type: data_type.to_string(),
            reports: reports.iter().map(|f| f.to_string()).collect(),
        });
    };

    let missing_dupe = |report: &str, version: i8| {
        println!("Patching report {report} --> {report}v{version} (dupe)");
        dictionary
            .lock()
            .unwrap()
            .iter_mut()
            .filter(|f| f.reports.contains(&report.to_string()))
            .for_each(|f| f.reports.push(format!("{report}v{version}")));
    };

    // Fields missing reports
    report_uses("flow_direction", "INT715A");
    report_uses("flow_direction", "INT715B");
    report_uses("trn", "INT705v2");
    report_uses("trn", "INT705v3");
    report_uses("trn_priority", "INT706v2");
    report_uses("gas_date", "INT653v3");

    // Dedupe missing v2 reports
    missing_dupe("INT718", 2);
    missing_dupe("INT656", 2);
    missing_dupe("INT657", 2);
    missing_dupe("INT653", 3);

    // Missing fields
    new_field("market_position", "varchar(10)", vec!["INT724"]); // 'Long' or 'Short'

    // All done :3
    dictionary.into_inner().unwrap()
}
