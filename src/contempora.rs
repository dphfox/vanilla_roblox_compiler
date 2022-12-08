use anyhow::Result;
use std::{collections::{HashMap, HashSet}, fs::File, io::{BufReader, BufRead, Write}, path::{PathBuf, Path}};

fn get_indent(string: &str) -> usize {
	let mut num_tabs = 0;
	loop {
		if string.starts_with(&"\t".repeat(num_tabs + 1)) || string.starts_with(&"    ".repeat(num_tabs + 1)){
			num_tabs += 1;
		} else {
			break;
		}
	}
	num_tabs
}

pub struct TagSystem {
	pub all_tags: HashSet<String>,
	pub tag_colours: HashMap<String, String>,

	pub instance_tags: HashMap<String, Vec<String>>,
	pub general_tags: HashMap<String, Vec<String>>
}

#[derive(serde::Serialize)]
struct TagSystemJSON {
	pub all_tags: Vec<String>,
	pub tag_colours: HashMap<String, String>,

	pub instance_tags: HashMap<String, Vec<String>>,
	pub general_tags: HashMap<String, Vec<String>>
}

impl TagSystem {
	pub fn new() -> Self {
		Self {
			all_tags: HashSet::new(),
			tag_colours: HashMap::new(),

			instance_tags: HashMap::new(),
			general_tags: HashMap::new()
		}
	}

	pub fn parse_tree_file(&mut self, filename: &Path, colour: String) -> Result<()> {
		let file = File::open(filename)?;
		let reader = BufReader::new(file);

		let mut last_indent;
		let mut curr_indent = 0;
		let mut tag_stack: Vec<String> = vec![];
		for line in reader.lines() {
			let line = line?;
			last_indent = curr_indent;
			curr_indent = get_indent(&line);

			let indent_difference: isize = (curr_indent as isize) - (last_indent as isize);

			if indent_difference < 0 {
				for _ in 0..indent_difference.abs() {
					tag_stack.pop();
				}
			}

			let line_trim = line.trim();

			if line_trim.starts_with(">>") {
				// general
				let icon_name = line_trim[2..].trim();
				self.general_tags.insert(icon_name.to_string(), tag_stack.clone());
			} else if line_trim.starts_with(">") {
				// instance
				let icon_name = line_trim[1..].trim();
				self.instance_tags.insert(icon_name.to_string(), tag_stack.clone());
			} else {
				// category
				tag_stack.push(line_trim.to_string());
				self.all_tags.insert(line_trim.to_string());
				if curr_indent == 0 {
					self.tag_colours.insert(line_trim.to_string(), colour.clone());
				}
			}
		}

		Ok(())
	}

	pub fn output(&self, filename: &Path) -> Result<()> {
		let mut sys_json = TagSystemJSON {
			all_tags: self.all_tags.iter().cloned().collect(),
			tag_colours: self.tag_colours.clone(),
			instance_tags: self.instance_tags.clone(),
			general_tags: self.general_tags.clone()
		};
		sys_json.all_tags.sort_unstable();
		let json = serde_json::to_string_pretty(&sys_json)?;
		let mut file = File::create(filename)?;
		write!(file, "{}", json)?;
		Ok(())
	}
}

pub fn do_tag_tree_parse() -> Result<()> {
	let mut sys = TagSystem::new();

	let paths = std::fs::read_dir("in/tag_trees")?;
	for path in paths {
		let path = path?;
		sys.parse_tree_file(&path.path(), path.path().file_stem().unwrap().to_str().unwrap().to_string())?;
	}

	sys.output(Path::new("in/tag_trees/generated.json"))?;

	Ok(())
}