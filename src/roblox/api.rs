use serde::Deserialize;

#[derive(Deserialize)]
pub struct APIDump {
    #[serde(rename = "Classes")]
    pub classes: Vec<APIClass>
}

#[derive(Deserialize)]
pub struct APIClass {
    #[serde(rename = "Name")]
    pub name: String
}