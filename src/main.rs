#![allow(dead_code)]

use usvg::*;
use anyhow::{Result};
use chrono::{Datelike, Timelike};
use std::{fs, path, collections::HashMap, rc::Rc, time::Instant};
use itertools::Itertools;
use tiny_skia::{Pixmap, Transform};
use rayon::prelude::*;

mod vanilla;
use vanilla::*;

fn main() -> Result<()> {
    let start_time = Instant::now();

    let working_directory = "E:/# High Speed Output/Vanilla/";
    
    print!("Loading icon pack mappings... ");
    let icon_mappings = IconPackMappings::from_file(path::Path::new("in/mappings.json"))?;
    println!("loaded {} mappings for api version {}.", icon_mappings.icons.len(), icon_mappings.api);
    
    print!("Preparing output directory... ");
    let now = chrono::Local::now();
    let root_out_dir = format!(
        "{}/out/{:04}-{:02}-{:02}_at_{:02}-{:02}",
        working_directory, now.year(), now.month(), now.day(), now.hour(), now.minute()
    );
    fs::create_dir_all(&root_out_dir)?;
    println!("done.");

    print!("Calculating unique icon subset... ");
    let unique_icons = icon_mappings.icons.par_iter()
        .map(|(_, icon_name)| icon_name.clone())
        .collect::<Vec<String>>();
    let unique_icons = unique_icons.into_iter().unique().collect::<Vec<String>>();
    println!("done.");

    print!("Loading icon vector data... ");
    let icon_datas = unique_icons.into_par_iter()
        .map(|icon_name| {
            let icon_path = format!("in/icons/{}.svg", icon_name);
            let svg_data = fs::read(icon_path)?;
            let icon_data = IconData::from_svg_data(svg_data)?;
            Ok((icon_name, icon_data))
        })
        .collect::<Result<HashMap<String, IconData>>>()?;
    println!("done.");

    println!("Started parallel compilation of icon packs.");

    let icon_palettes = fs::read_dir("in/palettes")?.par_bridge()
        .map(|entry| anyhow::Ok(IconPackPalette::from_file(&entry?.path())?));
    
    icon_palettes.map(|palette| {
        let palette = palette?;

        /*
            Preparing output directory
        */
        let palette_out_dir = format!("{}/{}", root_out_dir, palette.name);
        let palette_icons_out_dir = format!("{}/icons", palette_out_dir);
        fs::create_dir(&palette_out_dir)?;
        fs::create_dir(&palette_icons_out_dir)?;

        /*
            Combining instance icons with palette colours
        */
        let mappings_with_colour = icon_mappings.icons.par_iter()
            .map(|(instance_name, icon_name)| {
                let icon_colour = palette.icons.get(instance_name).unwrap_or(&palette.icon_default);
                (instance_name, icon_name, icon_colour)
            });
        
        /*
            Finding all unique combos of icons/colours used, for efficiency
        */
        let unique_icons_colours = mappings_with_colour.clone()
            .map(|(_, icon_name, icon_colour)| (icon_name, icon_colour))
            .collect::<Vec<(&String, &String)>>();
        let unique_icons_colours = unique_icons_colours.into_iter().unique().collect::<Vec<(&String, &String)>>();
        
        /*
            Generating spritesheet icon IDs for instances
        */
        let instance_icon_ids = mappings_with_colour
            .map(|(instance_name, icon_name, icon_colour)| {
                let icon_id = unique_icons_colours.par_iter()
                    .position_any(|(unique_name, unique_colour)| *unique_name == icon_name && *unique_colour == icon_colour)
                    .unwrap();
                (instance_name, icon_id)
            });

        /*
            Writing spritesheet icon IDs to file
        */
        let icon_id_out_file = format!("{}/icon_ids.txt", palette_out_dir);
        let icon_id_out_contents = instance_icon_ids
            .map(|(instance_name, icon_id)| format!("{}: {}\n", instance_name, icon_id))
            .reduce(|| String::new(), |line_a, line_b| format!("{}{}", line_a, line_b));
        fs::write(icon_id_out_file, icon_id_out_contents)?;

        /*
            Rendering icons
        */
        IconPackTheme::ALL_THEMES.par_iter().map(|theme| {
            let theme_name = match theme {
                IconPackTheme::Graphite => "graphite",
                IconPackTheme::Platinum => "platinum"
            };
            let theme_icons_out_dir = format!("{}/{}", palette_icons_out_dir, theme_name);
            fs::create_dir(&theme_icons_out_dir)?;

            let theme_colour_definitions = palette.definitions.get(&theme)
                .ok_or(anyhow::anyhow!("No such theme {:?} in palette {}", theme, palette.name))?;
            let overlay_colour = theme_colour_definitions.get(&palette.icon_overlay)
                .ok_or(anyhow::anyhow!("No such overlay colour {} in palette {}", &palette.icon_overlay, palette.name))?;
                    
            unique_icons_colours.par_iter().enumerate()
                .map(|(icon_id, (icon_name, icon_colour))| {
                    let icon_data = icon_datas.get(*icon_name).ok_or(anyhow::anyhow!("No icon data for {}", icon_name))?;
                    
                    let main_layer_colour = theme_colour_definitions.get(*icon_colour)
                        .ok_or(anyhow::anyhow!("No such colour {} in palette {}", icon_colour, palette.name))?;

                    let icon_layer_fills = IconLayerFills::from_colours(IconLayerColours::Duo {
                        main: *main_layer_colour,
                        overlay: *overlay_colour,
                        fallback_opacity: palette.duotone_fallback_opacity
                    });

                    let svg_tree = Tree::create(Svg {
                        size: Size::new(16.0, 16.0).ok_or(anyhow::anyhow!("Failed to make svg size"))?,
                        view_box: ViewBox {
                            rect: Rect::new(0.0, 0.0, 16.0, 16.0).ok_or(anyhow::anyhow!("Failed to make viewbox"))?,
                            aspect: AspectRatio::default()
                        }
                    });
                    if let Some(secondary_data) = &icon_data.secondary_path_data {
                        svg_tree.root().append_kind(NodeKind::Path(Path {
                            fill: Some(icon_layer_fills.front_layer),
                            data: Rc::new(icon_data.primary_path_data.clone()),
                            .. Path::default()
                        }));
                        svg_tree.root().append_kind(NodeKind::Path(Path {
                            fill: Some(icon_layer_fills.back_layer),
                            data: Rc::new(secondary_data.clone()),
                            .. Path::default()
                        }));
                    } else {
                        svg_tree.root().append_kind(NodeKind::Path(Path {
                            fill: Some(icon_layer_fills.single_layer),
                            data: Rc::new(icon_data.primary_path_data.clone()),
                            .. Path::default()
                        }));
                    }

                    let mut pixmap_duo = Pixmap::new(16, 16).ok_or(anyhow::anyhow!("Failed to make pixmap"))?;
                    resvg::render(&svg_tree, FitTo::Original, Transform::default(), pixmap_duo.as_mut()).ok_or(anyhow::anyhow!("Failed to render"))?;
                    let render_path = format!("{}/explorer-icon-{}.png", theme_icons_out_dir, icon_id);
                    pixmap_duo.save_png(render_path)?;

                    Ok(())
                })
                .reduce(|| anyhow::Ok(()), |accum, this| if accum.is_ok() {this} else {accum})?;
            Ok(())
        })
        .reduce(|| anyhow::Ok(()), |accum, this| if accum.is_ok() {this} else {accum})?;
        Ok(())
    })
    .reduce(|| anyhow::Ok(()), |accum, this| if accum.is_ok() {this} else {accum})?;
    let time_since_start = start_time.elapsed();

    println!("Completed in {} milliseconds.", time_since_start.as_millis());

    Ok(())
}
