//! The catalog is COMPONENT_MATRIX.md itself: parsed once at startup so the
//! in-app statuses and the document can never disagree.

use std::sync::OnceLock;

const SOURCE: &str = include_str!("../COMPONENT_MATRIX.md");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Status {
    Working,
    Partial,
    Broken,
    NotApplicable,
    Untested,
}

impl Status {
    fn parse(value: &str) -> Status {
        match value.trim() {
            "WORKING" => Status::Working,
            "PARTIAL" => Status::Partial,
            "BROKEN" => Status::Broken,
            "NOT_APPLICABLE" => Status::NotApplicable,
            _ => Status::Untested,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Status::Working => "WORKING",
            Status::Partial => "PARTIAL",
            Status::Broken => "BROKEN",
            Status::NotApplicable => "N/A",
            Status::Untested => "UNTESTED",
        }
    }
}

#[derive(Debug)]
pub struct Row {
    pub category: &'static str,
    pub component: &'static str,
    pub included: bool,
    pub status: Status,
    pub screen: &'static str,
    pub notes: &'static str,
}

pub fn rows() -> &'static [Row] {
    static ROWS: OnceLock<Vec<Row>> = OnceLock::new();
    ROWS.get_or_init(|| {
        let mut rows = Vec::new();
        let mut category = "";
        for line in SOURCE.lines() {
            if let Some(heading) = line.strip_prefix("## ") {
                category = heading.trim();
                continue;
            }
            if !line.starts_with("| ") || line.starts_with("| Component") {
                continue;
            }
            let cells: Vec<&'static str> =
                line.trim_matches('|').split('|').map(str::trim).collect();
            if cells.len() < 5 {
                continue;
            }
            rows.push(Row {
                category,
                component: cells[0],
                included: cells[1] == "Yes",
                status: Status::parse(cells[2]),
                screen: cells[3],
                notes: cells[4],
            });
        }
        rows
    })
}

pub fn categories() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for row in rows() {
        if !out.contains(&row.category) {
            out.push(row.category);
        }
    }
    out
}

pub fn count(status: Status) -> usize {
    rows().iter().filter(|row| row.status == status).count()
}

/// Rows demonstrated on one screen, for its header.
pub fn for_screen(screen: &str) -> impl Iterator<Item = &'static Row> + '_ {
    rows().iter().filter(move |row| row.screen == screen)
}
