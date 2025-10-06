use usvg::*;
use anyhow::{Context, Result};
use chrono::{Datelike, Timelike};
use std::{collections::HashMap, fs::{self, File}, rc::Rc, time::Instant};
use itertools::Itertools;
use tiny_skia::{Pixmap, Transform};
use rayon::prelude::*;

use crate::{vanilla::*, contempora};

fn render_icon(icon_data: &IconData, fills: IconFills, size_u32: u32) -> Result<Pixmap> {
	let svg_tree = Tree::create(Svg {
		size: Size::new(16.0, 16.0).ok_or(anyhow::anyhow!("Failed to make svg size"))?,
		view_box: ViewBox {
			rect: Rect::new(0.0, 0.0, 16.0, 16.0).ok_or(anyhow::anyhow!("Failed to make viewbox"))?,
			aspect: AspectRatio::default()
		}
	});
	if let Some(primary_data) = &icon_data.primary_path_data {
		svg_tree.root().append_kind(NodeKind::Path(Path {
			fill: Some(fills.primary_fill),
			data: Rc::new(primary_data.clone()),
			.. Path::default()
		}));
	}
	if let Some(secondary_data) = &icon_data.secondary_path_data {
		svg_tree.root().append_kind(NodeKind::Path(Path {
			fill: Some(fills.secondary_fill),
			data: Rc::new(secondary_data.clone()),
			.. Path::default()
		}));
	}
	if let Some(overlay_data) = &icon_data.overlay_path_data {
		svg_tree.root().append_kind(NodeKind::Path(Path {
			fill: Some(fills.overlay_fill),
			data: Rc::new(overlay_data.clone()),
			.. Path::default()
		}));
	}

	let mut pixmap = Pixmap::new(size_u32, size_u32).ok_or(anyhow::anyhow!("Failed to make pixmap"))?;
	resvg::render(&svg_tree, FitTo::Size(size_u32, size_u32), Transform::default(), pixmap.as_mut()).ok_or(anyhow::anyhow!("Failed to render"))?;
	Ok(pixmap)
}

fn get_fill_for<'a>(tag_fills: &'a HashMap<String, IconFills>, _tag_system: &contempora::TagSystem, tag: &str) -> Option<&'a IconFills> {
	let mut fill = None;
	let mut partial_tag = String::new();
	for part in tag.split(">") {
		partial_tag = if partial_tag.is_empty() { part.to_owned() } else { format!("{}>{}", partial_tag, part) };
		if let Some(partial_tag_fill) = tag_fills.get(partial_tag.trim()) {
			fill = Some(partial_tag_fill);
		}
	}

	fill
}

pub fn do_icon_compile() -> Result<()> {
	let working_directory = ".";
	let root_out_dir = {
		let now = chrono::Local::now();
		format!(
			"{}/out/{:04}-{:02}-{:02}_at_{:02}-{:02}",
			working_directory, now.year(), now.month(), now.day(), now.hour(), now.minute()
		)
	};

	let start_time = Instant::now();

	/*
		Loading icon tags
	*/
	print!("Loading icon tags... ");
	let tag_system = contempora::TagSystem::from_file(File::open("in/tags.json")?)?;
	println!("loaded.");

	for concern in tag_system.lint() {
		println!("-> (lint) {}", concern);
	}

	/*
		Reflection metadata generation
	*/
	print!("Loading in reflection metadata file... ");
	let reflection_in = std::fs::read_to_string("in/ReflectionMetadata.xml")?;
	let reflection_in = reflection_in.split("\n").filter(|x| return !x.contains("<string name=\"ExplorerOrder\">")).collect_vec();
	let mut reflection_out: Vec<String> = vec![];
	println!("done.");

	enum State {
		Ready,
		InItem,
		InProperties
	}

	let mut state = State::Ready;
	for line in reflection_in.into_iter() {
		reflection_out.push(line.to_string());
		match state {
			State::Ready => {
				if line.contains("<Item class=\"ReflectionMetadataClass\">") {
					state = State::InItem;
				} else {
					state = State::Ready;
				}
			},
			State::InItem => {
				if line.contains("<Properties>") {
					state = State::InProperties;
				} else {
					state = State::Ready;
				}
			},
			State::InProperties => {
				const OPEN_SUBSTR: &str = "<string name=\"Name\">";
				const CLOSE_SUBSTR: &str = "</string>";
				let tag_start_idx = line.find(OPEN_SUBSTR);
				if let Some(tag_start_idx) = tag_start_idx {
					let tag_start_idx = tag_start_idx + OPEN_SUBSTR.len();
					let tag_end_idx = line.find(CLOSE_SUBSTR);
					if let Some(tag_end_idx) = tag_end_idx {
						let name = line[tag_start_idx..tag_end_idx].to_string();
						println!("-> {name}");
						if let Some(tag_list) = tag_system.instance_tags.get(&name) {
							let first_tag = tag_list.first();
							if let Some(first_tag) = first_tag {
								let order = tag_system.all_tags.iter().position(|x| x == first_tag);
								if let Some(order) = order {
									println!("bonk");
									let out = format!("<string name=\"ExplorerOrder\">{order}</string>");
									reflection_out.push(out);
								}
							}
						}
					}
				}
				state = State::Ready;
			}
		}
		
	}

	print!("Writing final reflection metadata file... ");
	fs::create_dir_all(&root_out_dir)?;
	std::fs::write(format!("{}/ReflectionMetadata.xml", root_out_dir), reflection_out.join("\n"))?;
	println!("done.");

	/*
		Loading icon pack mappings
	*/
	print!("Loading icon pack mappings... ");
	let icon_mappings = IconMappings::new_from_file(std::path::Path::new("in/mappings.json"))?;
	println!("loaded.");

	for (category_name, category_mappings) in icon_mappings.icons.iter() {
		println!("-> {} {} icon mappings", category_mappings.len(), category_name);
	}

	/*
		Checking for missing icons
	*/
	print!("Checking for missing icons... ");
	if let Ok(api) = crate::roblox::get_api_dump() {
		let mut missing_icons = vec![];
		let instance_icon_mappings = icon_mappings.icons.get("instance").expect("No instance icon mappings defined");
		for class in api.classes {
			if !instance_icon_mappings.contains_key(&class.name) {
				missing_icons.push(class.name);
			}
		}
		missing_icons.sort();
		println!("{} missing icons total.", missing_icons.len());
		for icon in missing_icons {
			println!("-> Missing: {}", &icon)
		}
	} else {
		println!("Couldn't connect.")
	}
	
	/*
		Discovering icons to be loaded
	*/
	print!("Discovering icons to be loaded... ");
	let unique_icons = icon_mappings.icons.par_iter()
	.map(|(_, category_icons)|{
		category_icons.into_par_iter()
		.map(|(_, icon_name)| icon_name.clone())
	})
	.flatten()
	.collect::<Vec<_>>();
	let unique_icons = unique_icons.into_iter().unique().collect::<Vec<String>>();
	println!("done.");

	/*
		Loading icon vector data
	*/
	print!("Loading icon vector data... ");
	let icon_datas =  unique_icons.par_iter()
	.map(|icon_name| {
		let icon_path = format!("in/icons/Vanilla/Icon={}.svg", icon_name);
		let svg_data = fs::read(icon_path).or(Err(anyhow::anyhow!("Could not find icon {}", &icon_name)))?;
		let icon_data = IconData::from_svg_data(svg_data).context(icon_name.clone())?;
		Ok((icon_name.clone(), icon_data))
	})
	.collect::<Result<HashMap<String, IconData>>>()?;
	println!("done.");

	/*
		Loading icon palette definitions
	*/
	print!("Loading icon palette definitions... ");
	let themed_icon_palettes = fs::read_dir("in/palettes")?.par_bridge()
	.map(|entry| anyhow::Ok(IconPalette::all_from_file(&entry?.path())?))
	.collect::<Result<Vec<_>>>()?;
	println!("done.");

	/*
		Compiling palettes
	*/
	print!("Compiling {} palettes in {} themes... ", themed_icon_palettes.len(), IconTheme::ALL_THEMES.len());
	themed_icon_palettes.into_par_iter()
	.map(|themed_icon_palette| {
		themed_icon_palette.into_par_iter()
		.map(|(theme, icon_palette)| {
			let theme_name = match theme {
				IconTheme::Light => "Light",
				IconTheme::Dark => "Dark"
			};

			let palette_out_dir = format!(
				"{}/{}/{}/RobloxCustom", root_out_dir, icon_palette.name, theme_name
			);
			fs::create_dir_all(&palette_out_dir)?;
			fs::copy("in/index.theme", format!("{}/index.theme", palette_out_dir))?;

			/*
				Rendering all icons
			*/
			icon_mappings.icons.par_iter()
			.map(|(category_name, category_mappings)| {
				let icon_scales = icon_mappings.scaling.get(category_name)
				.ok_or(anyhow::anyhow!("No scaling specified for category {}", category_name))?;

				icon_scales.par_iter()
				.map(|scaling| {
					let scaled_icon_dir = format!("{}/{}/{}x/{}", palette_out_dir, category_name, scaling.size, (scaling.scale * 100.0) as u32);
					Ok(fs::create_dir_all(&scaled_icon_dir)?)
				})
				.collect::<Result<()>>()?;

				category_mappings.par_iter()
				.map(|(file_name, icon_name)| {
					let icon_data = icon_datas.get(icon_name)
					.ok_or(anyhow::anyhow!("{}/{} uses nonexistent icon {}", category_name, file_name, icon_name))?;

					let mut fills = &icon_palette.default_fills;

					if category_name == "instance" {
						if let Some(tag_list) = tag_system.instance_tags.get(file_name) {
							if let Some(tag_fill) = tag_list.iter().filter_map(|tag| get_fill_for(&icon_palette.tag_fills, &tag_system, tag)).next() {
								fills = tag_fill;
							}
						}
					} else if category_name == "general" {
						if let Some(tag_list) = tag_system.general_tags.get(file_name) {
							if let Some(tag_fill) = tag_list.iter().filter_map(|tag| get_fill_for(&icon_palette.tag_fills, &tag_system, tag)).next() {
								fills = tag_fill;
							}
						}
					}

					icon_scales.par_iter()
					.map(|scaling| {
						let scaled_icon_dir = format!("{}/{}/{}x/{}", palette_out_dir, category_name, scaling.size, (scaling.scale * 100.0) as u32);
						let scaled_icon_path = format!("{}/{}.png", scaled_icon_dir, file_name);
						let pixel_size = (scaling.size as f32 * scaling.scale) as u32;
						let pixmap = render_icon(icon_data, fills.clone(), pixel_size)?;
						pixmap.save_png(scaled_icon_path)?;
						Ok(())
					})
					.collect::<Result<()>>()
				})
				.collect::<Result<()>>()
			})
			.collect::<Result<()>>()
		})
		.collect::<Result<()>>()
	})
	.collect::<Result<()>>()?;
	println!("done.");

	let time_since_start = start_time.elapsed();
	println!("Completed in {} milliseconds.", time_since_start.as_millis());
	Ok(())
}