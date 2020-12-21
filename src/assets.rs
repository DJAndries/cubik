use std::path::{Path, PathBuf};

const PREFIXES: [(&'static str, bool); 5] = [
	("examples", false),
	("..", false),
	("../..", false),
	("../../..", false),
	("/usr/share/", true)
];

pub fn find_asset(path: &str, app_id: &str) -> PathBuf {
	let path = Path::new(path);

	if !path.exists() {
		for (prefix, add_id) in &PREFIXES {
			let mut prefix = String::from(*prefix);
			if *add_id {
				prefix.push_str(app_id);
			}
			let new_path = Path::new(&prefix).join(path);
			if new_path.exists() {
				return new_path;
			}
		}
	}

	path.to_path_buf()
}
