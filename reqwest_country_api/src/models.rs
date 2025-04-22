use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Country {
    pub name: CountryName,
    pub tld: Option<Vec<String>>,
    pub cca2: String,
    pub ccn3: String,
    pub cca3: String,
    pub cioc: Option<String>,
    pub independent: Option<bool>,
    pub status: String,
    pub unMember: bool,
    pub currencies: Option<HashMap<String, Currency>>,
    pub idd: Idd,
    pub capital: Option<Vec<String>>,
    pub altSpellings: Vec<String>,
    pub region: String,
    pub subregion: Option<String>,
    pub languages: Option<HashMap<String, String>>,
    pub translations: Option<HashMap<String, Translation>>,
    pub latlng: Vec<f64>,
    pub landlocked: bool,
    pub borders: Option<Vec<String>>,
    pub area: f64,
    pub demonyms: Option<HashMap<String, Demonym>>,
    pub flag: String,
    pub maps: Maps,
    pub population: u64,
    pub gini: Option<HashMap<String, f64>>,
    pub fifa: Option<String>,
    pub car: Car,
    pub timezones: Vec<String>,
    pub continents: Vec<String>,
    pub flags: ImageUrls,
    pub coatOfArms: ImageUrls,
    pub startOfWeek: String,
    pub capitalInfo: Option<CapitalInfo>,
    pub postalCode: Option<PostalCode>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CountryName {
    pub common: String,
    pub official: String,
    pub nativeName: Option<HashMap<String, NativeName>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NativeName {
    pub official: String,
    pub common: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Currency {
    pub name: String,
    pub symbol: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Idd {
    pub root: Option<String>,
    pub suffixes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Translation {
    pub official: String,
    pub common: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Demonym {
    pub  f: String,
    pub m: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Maps {
    googleMaps: String,
    openStreetMaps: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Car {
    signs: Option<Vec<String>>,
    side: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ImageUrls {
    png: Option<String>,
    svg: Option<String>,
    alt: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CapitalInfo {
    latlng: Option<Vec<f64>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PostalCode {
    format: String,
    regex: Option<String>,
}