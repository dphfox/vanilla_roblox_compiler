use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use hex_color::HexColor;
use std::fs;
use std::path;

use crate::vanilla::*;

#[derive(Serialize, Deserialize)]
struct IconPackPaletteJSON {
    definitions: HashMap<String, HashMap<String, String>>,
    icon_default: String,
    icon_overlay: String,
    icons: HashMap<String, String>
}

impl IconPackPalette {
    fn from_json(json: IconPackPaletteJSON) -> Result<Self> {
        let parsed_defs = json.definitions.iter()
            .map(|(def_name, def)| {
                let parsed_def = def.iter()
                    .map(|(theme_name, color_hex)| {
                        let theme = match theme_name.as_str() {
                            "platinum" => IconPackTheme::Platinum,
                            "graphite" => IconPackTheme::Graphite,
                            _ => anyhow::bail!("Invalid theme string")
                        };
                        let color_parsed: HexColor = color_hex.parse()?;
                        let colour = Color::new_rgb(color_parsed.r, color_parsed.g, color_parsed.b);

                        Ok((theme, colour))
                    })
                    .collect::<Result<HashMap<IconPackTheme, Color>>>()?;
                Ok((def_name.clone(), parsed_def))
            })
            .collect::<Result<HashMap<String, HashMap<IconPackTheme, Color>>>>()?;

        Ok(Self {
            definitions: parsed_defs,
            icon_default: json.icon_default,
            icon_overlay: json.icon_overlay,
            icons: json.icons
        })
    }

    pub fn from_file(path: &path::Path) -> Result<Self> {
        Self::from_json(serde_json::from_slice(fs::read(path)?.as_slice())?)
    }
}

#[derive(Serialize, Deserialize)]
struct IconPackMappingsJSON {
    api: u32,
    icons: HashMap<String, String>
}

impl IconPackMappings {
    fn from_json(json: IconPackMappingsJSON) -> Result<Self> {
        Ok(Self {
            api: json.api,
            icons: json.icons
        })
    }

    pub fn from_file(path: &path::Path) -> Result<Self> {
        Self::from_json(serde_json::from_slice(fs::read(path)?.as_slice())?)
    }
}