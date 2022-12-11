
use anyhow::Result;
use usvg::*;
use std::collections::HashMap;

mod vanilla_serde;

pub struct IconData {
    pub primary_path_data: Option<PathData>,
    pub secondary_path_data: Option<PathData>,
	pub overlay_path_data: Option<PathData>
}

impl IconData {
    pub fn from_svg_data(svg_data: Vec<u8>) -> Result<Self> {
        let tree = Tree::from_data(&svg_data, &usvg::Options::default().to_ref())?;

        let mut primary_data = None;
        let mut secondary_data = None;
		let mut overlay_data = None;

        for mut node in tree.root().descendants() {
            if let NodeKind::Path(ref mut path) = *node.borrow_mut() {
                let fill = path.fill.as_ref().ok_or(anyhow::anyhow!("No fill found for path"))?;
				let is_overlay = match fill.paint {
					Paint::Color(paint) => paint.blue < 200 && paint.red < 200,
					Paint::Link(_) => false
				};
                let is_secondary = fill.opacity.to_u8() < 200;

				if is_overlay {
					overlay_data = match overlay_data {
                        None => Some(path.data.as_ref().clone()),
                        Some(_) => anyhow::bail!("Icon has two overlay layers")
                    }
				} else if is_secondary {
                    secondary_data = match secondary_data {
                        None => Some(path.data.as_ref().clone()),
                        Some(_) => anyhow::bail!("Icon has two secondary layers")
                    }
                } else {
                    primary_data = match primary_data {
                        None => Some(path.data.as_ref().clone()),
                        Some(_) => anyhow::bail!("Icon has two primary layers")
                    }
                }
            }
        }

        Ok(Self {
            primary_path_data: primary_data,
            secondary_path_data: secondary_data,
			overlay_path_data: overlay_data
        })
    }
}
#[derive(Clone)]
pub struct IconFills {
    pub primary_fill: Fill,
    pub secondary_fill: Fill,
	pub overlay_fill: Fill
}

impl IconFills {
    pub fn from_colours(main: Color, overlay: Color) -> Self {
        Self {
            primary_fill: Fill {
                paint: Paint::Color(main),
                opacity: 1.0.into(),
                rule: FillRule::EvenOdd
            },
            secondary_fill: Fill {
                paint: Paint::Color(main),
                opacity: 0.6.into(),
                rule: FillRule::EvenOdd
            },
			overlay_fill: Fill {
                paint: Paint::Color(overlay),
                opacity: 1.0.into(),
                rule: FillRule::EvenOdd
            }
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum IconTheme {
    Light,
    Dark
}

impl IconTheme {
    pub const ALL_THEMES: [Self; 2] = [Self::Light, Self::Dark];
}

pub struct IconPalette {
    pub name: String,
    pub default_fills: IconFills,
    pub tag_fills: HashMap<String, IconFills>
}

pub struct IconMappings {
    pub scaling: HashMap<String, Vec<IconScaling>>,
    pub icons: HashMap<String, HashMap<String, String>>
}

pub struct IconScaling {
    pub size: u32,
    pub scale: f32
}