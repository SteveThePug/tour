use std::fs;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

const INFO_PATH: &str = "./.tour/info";

struct TourInfo {
    author: String,
    description: String,
    language: String,
    created: String,
    updated: String,
}

impl TourInfo {
    fn parse(content: &str) -> Self {
        let mut author = String::new();
        let mut description = String::new();
        let mut language = String::new();
        let mut created = String::new();
        let mut updated = String::new();

        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "author" => author = value.to_string(),
                    "description" => description = value.to_string(),
                    "language" => language = value.to_string(),
                    "created" => created = value.to_string(),
                    "updated" => updated = value.to_string(),
                    _ => {}
                }
            }
        }

        TourInfo { author, description, language, created, updated }
    }

    fn serialize(&self) -> String {
        format!(
            "author={}\ndescription={}\nlanguage={}\ncreated={}\nupdated={}\n",
            self.author, self.description, self.language, self.created, self.updated
        )
    }
}

pub fn set_info() -> Result<(), io::Error> {
    macro_rules! prompt {
        ($msg:expr) => {{
            print!($msg);
            io::stdout().flush()?;
            let mut buf = String::new();
            io::stdin().read_line(&mut buf)?;
            buf.trim().to_string()
        }};
    }

    let author = prompt!("Author: ");
    let description = prompt!("Description: ");
    let language = prompt!("Language: ");
    let today = current_date();

    let info = TourInfo {
        author,
        description,
        language,
        created: today.clone(),
        updated: today,
    };

    fs::write(INFO_PATH, info.serialize())
}

pub fn get_info() -> Result<(), io::Error> {
    let content = fs::read_to_string(INFO_PATH).map_err(|_| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "No tour info found. Run `tour init` to set up a tour.",
        )
    })?;
    let info = TourInfo::parse(&content);
    println!("Author:      {}", info.author);
    println!("Description: {}", info.description);
    println!("Language:    {}", info.language);
    println!("Created:     {}", info.created);
    println!("Updated:     {}", info.updated);
    Ok(())
}

pub fn update_last_modified() -> Result<(), io::Error> {
    let content = fs::read_to_string(INFO_PATH).unwrap_or_default();
    let mut info = TourInfo::parse(&content);
    info.updated = current_date();
    fs::write(INFO_PATH, info.serialize())
}

pub fn info() -> Result<(), crate::error::TourError> {
    crate::utils::require_tour()?;
    get_info()?;
    Ok(())
}

fn current_date() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (y, m, d) = days_to_ymd((secs / 86400) as u32);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn days_to_ymd(mut days: u32) -> (u32, u32, u32) {
    let mut year = 1970u32;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let month_days = if is_leap(year) {
        [31u32, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31u32, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u32;
    for (i, &md) in month_days.iter().enumerate() {
        if days < md {
            month = i as u32 + 1;
            break;
        }
        days -= md;
    }
    (year, month, days + 1)
}

fn is_leap(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}
