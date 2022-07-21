#![allow(dead_code)]

use anyhow::{Result, Context};
use chrono::{Datelike, Timelike};
use std::{fs, path, collections::HashMap};
use itertools::Itertools;
use usvg::Color;

mod vanilla;
use vanilla::*;

fn main() -> Result<()> {
    print!("Loading icon pack palettes... ");
    let icon_palettes = fs::read_dir("in/palettes")?
        .map(|entry| Ok(IconPackPalette::from_file(&entry?.path())?))
        .collect::<Result<Vec<IconPackPalette>>>()?;
    println!("loaded {} palettes.", icon_palettes.len());

    print!("Loading icon pack mappings... ");
    let icon_mappings = IconPackMappings::from_file(path::Path::new("in/mappings.json"))?;
    println!("loaded {} mappings for api version {}.", icon_mappings.icons.len(), icon_mappings.api);
    
    print!("Collecting icons used by mappings... ");
    let used_icons = icon_mappings.icons.iter().map(|(_, icon_name)| icon_name)
        .unique().collect::<Vec<&String>>();
    println!(" found {} unique icons.", used_icons.len());

    print!("Converting mappings to numeric IDs... ");
    let icon_id_mappings = icon_mappings.icons.iter()
        .map(|(instance_name, icon_name)| {
            let icon_id = used_icons.iter().position(|used_name| *used_name == icon_name).unwrap();
            (instance_name, icon_id)
        })
        .collect::<HashMap<&String, usize>>();
    println!("done.");

    print!("Preparing output directory... ");
    let now = chrono::Local::now();
    let out_dir = format!("out/{:04}-{:02}-{:02}_at_{:02}-{:02}", now.year(), now.month(), now.day(), now.hour(), now.minute());
    let icon_out_dir = format!("{}/icons", out_dir);
    fs::create_dir_all(&out_dir)?;
    fs::create_dir(&icon_out_dir)?;
    println!("done.");

    print!("Saving icon ID mappings...");
    let icon_id_out_file = format!("{}/icon_ids.txt", out_dir);
    let icon_id_out_contents = icon_id_mappings.iter().fold(String::new(), |text, (instance_name, icon_id)| format!("{}{}: {}\n", text, instance_name, icon_id));
    fs::write(icon_id_out_file, icon_id_out_contents)?;
    println!("done.");

    // print!("Flattening permutations... ");
    // let icon_pack_permutations = icon_palettes.iter().map(|palette| {
    //     IconPackLayerStyle::ALL_STYLES.iter().map(move |layer_style| {
    //         IconPackTheme::ALL_THEMES.iter().map(move |theme| {
    //             (palette, layer_style, theme)
    //         })
    //     })
    // }).flatten().flatten();
    // println!("done.");

    print!("Loading icon data... ");
    let icon_data = used_icons.iter()
        .map(|icon_name| {
            let icon_path = format!("in/icons/{}.svg", icon_name);
            let svg_data = fs::read(icon_path)?;
            let icon_data = IconData::from_svg_data(svg_data)?;
            Ok(icon_data)
        })
        .collect::<Result<Vec<IconData>>>()?;
    println!("done.");   

    print!("Rendering icons to PNG... ");

    let icon_fills = IconLayerFills::from_colours(IconLayerColours::Duo {
        main: Color::new_rgb(255, 170, 0),
        overlay: Color::new_rgb(255, 255, 255), 
        fallback_opacity: 0.75
    });
    println!("done.");

    Ok(())
}
