use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use hex_color::HexColor;
use std::fs;
use std::path;
use rayon::prelude::*;

use crate::vanilla::*;

#[derive(Serialize, Deserialize)]
struct IconPackPaletteJSON {
    name: String,
    definitions: HashMap<String, HashMap<String, String>>,
    icon_default: String,
    icon_overlay: String,
    single_tone_opacity: f64,
    duotone_fallback_opacity: f64,
    icons: HashMap<String, String>
}

impl IconPackPalette {
    fn from_json(json: IconPackPaletteJSON) -> Result<Self> {
        let definitions = json.definitions.par_iter()
            .map(|(theme_name, theme_colour_strings)| {
                let theme = match theme_name.as_str() {
                    "platinum" => IconPackTheme::Platinum,
                    "graphite" => IconPackTheme::Graphite,
                    _ => anyhow::bail!("Invalid theme string")
                };

                let theme_colours = theme_colour_strings.par_iter()
                    .map(|(colour_name, colour_string)| {
                        let colour_hex: HexColor = colour_string.parse()?;
                        let colour = Color::new_rgb(colour_hex.r, colour_hex.g, colour_hex.b);

                        Ok((colour_name.clone(), colour))
                    })
                    .collect::<Result<HashMap<String, Color>>>()?;
                    
                Ok((theme, theme_colours))
            })
            .collect::<Result<HashMap<IconPackTheme, HashMap<String, Color>>>>()?;

        Ok(Self {
            name: json.name,
            definitions: definitions,
            icon_default: json.icon_default,
            icon_overlay: json.icon_overlay,
            single_tone_opacity: json.single_tone_opacity,
            duotone_fallback_opacity: json.duotone_fallback_opacity,
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