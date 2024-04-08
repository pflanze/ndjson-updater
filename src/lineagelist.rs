
//! Parsing of metainformation related to pango lineages from
//! https://github.com/cov-lineages/lineages-website/raw/master/_data/lineage_data.full.json
//! (see https://cov-lineages.org/lineage_list.html)

//! Currently only 2 fields, since:

//! - There can be empty strings where dates are expected, need a custom parsing wrapper.
//! - Serde is increadibly slow on this for some reason (bug).
//! - Not all fields are needed anyway, currently.

//use chrono::NaiveDate;
use kstring::KString;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, serde::Deserialize)]
pub struct Count {
    pub date: KString, //XX  NaiveDate,
    pub count: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct CountryCounts {
    pub country: KString,
    pub counts: Vec<Count>
}

#[derive(Debug, serde::Deserialize)]
pub struct DateCount {
    pub date: KString, // XX NaiveDate,
    pub count: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct Lineage {
    #[serde(rename = "Lineage")] 
    pub lineage: KString,
    // #[serde(rename = "Countries")] 
    // pub countries: String,
    // #[serde(rename = "Country counts")]
    // pub country_counts: Vec<CountryCounts>,
    // #[serde(rename = "Earliest date")]
    // pub earliest_date: String, // XX handle "" please, OptionalNaiveDate,
    // #[serde(rename = "Latest date")]
    // pub latest_date: String, // XX handle "" please, OptionalNaiveDate,
    // #[serde(rename = "Number designated")]
    // pub number_designated: u32, // ?
    // #[serde(rename = "Number assigned")] 
    // pub number_assigned: u32, // ?
    // #[serde(rename = "Date")]
    // pub date: Vec<DateCount>,
    // #[serde(rename = "Travel history")]
    // pub travel_history: String,
    #[serde(rename = "Description")]
    pub description: String,
}

lazy_static!{
    static ref ALIASOF_RE: Regex = Regex::new(r"\b[Aa]lias +of +([A-Z]+(?:\.\d+)*)").unwrap();
}


impl Lineage {
    /// Alias information extracted from the `Description` text
    /// field. Not 100% reliable.
    pub fn get_alias_of(&self) -> Option<&str> {
        let cap = ALIASOF_RE.captures(&self.description)?;
        let mut caps = cap.iter();
        caps.next();
        caps.next().map(|c| c.unwrap().as_str())
    }
}
