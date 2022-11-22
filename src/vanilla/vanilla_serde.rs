use anyhow::Result;
use itertools::Itertools;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use hex_color::HexColor;
use std::fs;
use std::path;

use crate::vanilla::*;

#[derive(Serialize, Deserialize)]
struct IconPaletteJSON {
    name: String,

    theme_definitions: HashMap<String, HashMap<String, String>>,

    default_colour: String,
    duo_colour: String,

    icon_colours: HashMap<String, HashMap<String, String>>
}

impl IconPalette {
    fn all_from_json(json: IconPaletteJSON) -> Result<HashMap<IconTheme, Self>> {
        json.theme_definitions.into_iter()
        .map(|(theme_name, definitions)| {
            let theme = match theme_name.as_str() {
                "light" => IconTheme::Light,
                "dark" => IconTheme::Dark,
                other => anyhow::bail!("Invalid theme {} for palette {}", other, json.name)
            };

            let parsed_definitions = definitions.into_iter()
            .map(|(key, value_str)| {
                let value_hex: HexColor = value_str.parse()?;
                let value_colour = Color::new_rgb(value_hex.r, value_hex.g, value_hex.b);
                Ok((key, value_colour))
            })
            .collect::<Result<HashMap<_, _>>>()?;

			let duo_colour = parsed_definitions.get(&json.duo_colour)
			.ok_or(anyhow::anyhow!("Duo colour {} is not defined for palette {}", &json.duo_colour, json.name))?;

            let colour_fills = parsed_definitions.iter()
			.map(|(key, colour)| {
				Ok((key, IconFills::from_colours(*colour, *duo_colour)))
			})
			.collect::<Result<HashMap<_, _>>>()?;

            let default_fills = colour_fills.get(&json.default_colour)
			.ok_or(anyhow::anyhow!("Default colour {} is not defined for palette {}", json.default_colour, json.name))?
			.clone();

            let icon_fills = json.icon_colours.iter()
            .map(|(category_name, category_colours)| {
                let category_fills = category_colours.iter()
                .map(|(instance_name, colour_def)| {
                    let instance_fill = colour_fills.get(&colour_def)
                    .ok_or(anyhow::anyhow!("Colour {} for {} is not defined for palette {}", colour_def, instance_name, json.name))?;
                    Ok((instance_name.clone(), instance_fill.clone()))
                })
                .collect::<Result<HashMap<_, _>>>()?;

                Ok((category_name.clone(), category_fills))
            })
            .collect::<Result<HashMap<_, _>>>()?;

            let palette = Self {
                name: json.name.clone(),
                default_fills,
                icon_fills,
            };

            Ok((theme, palette))
        })
        .collect::<Result<HashMap<_,_>>>()
    }

    pub fn all_from_file(path: &path::Path) -> Result<HashMap<IconTheme, Self>> {
        Self::all_from_json(serde_json::from_slice(fs::read(path)?.as_slice())?)
    }
}

#[derive(Serialize, Deserialize)]
struct IconMappingsJSON {
    scaling: HashMap<String, Vec<IconScalingJSON>>,
    icons: HashMap<String, HashMap<String, String>>
}

#[derive(Serialize, Deserialize)]
struct IconScalingJSON {
    size: u32,
    scale: f32
}

impl From<IconScalingJSON> for IconScaling {
    fn from(json: IconScalingJSON) -> Self {
        Self {
            size: json.size,
            scale: json.scale
        }
    }
}

impl IconMappings {
    fn new_from_json(json: IconMappingsJSON) -> Result<Self> {
        Ok(Self {
            icons: json.icons,
            scaling: json.scaling.into_iter()
            .map(|(category_name, scales_json)| {
                (category_name, scales_json.into_iter().map_into().collect())
            })
            .collect()
        })
    }

    pub fn new_from_file(path: &path::Path) -> Result<Self> {
        Self::new_from_json(serde_json::from_slice(fs::read(path)?.as_slice())?)
    }
}