
use anyhow::Result;
use usvg::*;
use std::collections::HashMap;

mod vanilla_serde;
pub enum IconLayerColours {
    Single {
        colour: Color,
        back_opacity: f64
    },
    Duo {
        main: Color,
        overlay: Color,
        fallback_opacity: f64
    }
}

pub struct IconLayerFills {
    pub single_layer: Fill,
    pub front_layer: Fill,
    pub back_layer: Fill
}

impl IconLayerFills {
    pub fn from_colours(colours: IconLayerColours) -> Self {
        match colours {
            IconLayerColours::Single {colour, back_opacity} => Self {
                single_layer: Fill {
                    paint: Paint::Color(colour),
                    opacity: 1.0.into(),
                    rule: FillRule::EvenOdd
                },
                front_layer: Fill {
                    paint: Paint::Color(colour),
                    opacity: 1.0.into(),
                    rule: FillRule::EvenOdd
                },
                back_layer: Fill {
                    paint: Paint::Color(colour),
                    opacity: back_opacity.into(),
                    rule: FillRule::EvenOdd
                }
            },

            IconLayerColours::Duo {main, overlay, fallback_opacity} => Self {
                single_layer: Fill {
                    paint: Paint::Color(main),
                    opacity: 1.0.into(),
                    rule: FillRule::EvenOdd
                },
                front_layer: Fill {
                    paint: Paint::Color(overlay),
                    opacity: 1.0.into(),
                    rule: FillRule::EvenOdd
                },
                back_layer: Fill {
                    paint: Paint::Color(main),
                    opacity: if main == overlay {fallback_opacity.into()} else {1.0.into()},
                    rule: FillRule::EvenOdd
                }
            }
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum IconPackTheme {
    Platinum,
    Graphite
}

impl IconPackTheme {
    pub const ALL_THEMES: [Self; 2] = [Self::Platinum, Self::Graphite];
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum IconPackLayerStyle {
    SingleTone,
    Duotone
}

impl IconPackLayerStyle {
    pub const ALL_STYLES: [Self; 2] = [Self::SingleTone, Self::Duotone];
}

pub struct IconPackPalette {
    pub name: String,
    pub definitions: HashMap<IconPackTheme, HashMap<String, Color>>,
    pub icon_default: String,
    pub icon_overlay: String,
    pub single_tone_opacity: f64,
    pub duotone_fallback_opacity: f64,
    pub icons: HashMap<String, String>
}

pub struct IconPackMappings {
    pub api: u32,
    pub icons: HashMap<String, String>
}

pub struct IconData {
    pub primary_path_data: PathData,
    pub secondary_path_data: Option<PathData>
}

impl IconData {
    pub fn from_svg_data(svg_data: Vec<u8>) -> Result<Self> {
        let tree = Tree::from_data(&svg_data, &usvg::Options::default().to_ref())?;
    
        let mut primary_data = None;
        let mut secondary_data = None;
    
        for mut node in tree.root().descendants() {
            if let NodeKind::Path(ref mut path) = *node.borrow_mut() {
                let fill = path.fill.as_ref().ok_or(anyhow::anyhow!("No fill found for path"))?;
                let is_secondary = fill.opacity.to_u8() < 200;

                if is_secondary {
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
            primary_path_data: primary_data.ok_or(anyhow::anyhow!("Icon has no primary layer"))?,
            secondary_path_data: secondary_data
        })
    }
}