use std::{io::BufReader, fs::File, collections::HashMap};

use anyhow::Result;
use itertools::Itertools;
#[derive(serde::Deserialize)]
pub struct TagSystem {
	pub all_tags: Vec<String>,
	pub general_tags: HashMap<String, Vec<String>>,
	pub instance_tags: HashMap<String, Vec<String>>
}

impl TagSystem {
	pub fn from_file(file: File) -> Result<Self> {
		let reader = BufReader::new(file);
		let me = serde_json::from_reader(reader)?;
		Ok(me)
	}

	pub fn lint(&self) -> Vec<String> {
		let mut concerns = vec![];

		let used_tags = self.instance_tags.iter()
		.map(|(_, tag_list)| tag_list.iter())
		.flatten()
		.unique()
		.collect::<Vec<_>>();

		let extra_tags = self.instance_tags.iter()
		.map(|(_, tag_list)| tag_list.iter().filter(|tag| !self.all_tags.contains(tag)))
		.flatten()
		.unique();

		for extra_tag in extra_tags {
			concerns.push(format!("Tag '{}' was used, but does not appear in the `all_tags` list.", extra_tag));
		}

		let redundant_tags = self.all_tags.iter()
		.filter(|tag| !used_tags.contains(tag));

		for redundant_tag in redundant_tags {
			concerns.push(format!("Tag '{}' appears in the `all_tags` list, but isn't used anywhere.", redundant_tag));
		}

		concerns
	}
}