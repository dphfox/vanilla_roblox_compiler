const URL_API_DUMP: &str = "https://raw.githubusercontent.com/MaximumADHD/Roblox-Client-Tracker/roblox/API-Dump.json";

pub mod api;

pub fn get_api_dump() -> anyhow::Result<api::APIDump> {
    Ok(reqwest::blocking::get(URL_API_DUMP)?.json::<api::APIDump>()?)
}