use std::path::{Path, PathBuf};
use std::env::current_exe;

struct Prefix {
	path: &'static str,
	append_app_id: bool,
	relative_to_binary: bool
}

const PREFIXES: [Prefix; 6] = [
	Prefix { path: "examples", append_app_id: false, relative_to_binary: false },
	Prefix { path: "..", append_app_id: false, relative_to_binary: false },
	Prefix { path: "../..", append_app_id: false, relative_to_binary: false },
	Prefix { path: "../../..", append_app_id: false, relative_to_binary: false },
	Prefix { path: "/usr/lib", append_app_id: true, relative_to_binary: false },
	Prefix { path: "../Resources", append_app_id: false, relative_to_binary: true }
];

pub fn find_asset(path: &str, app_id: &str) -> PathBuf {
	let path = Path::new(path);

	if !path.exists() {
		for prefix in &PREFIXES {
			let mut new_path = if prefix.relative_to_binary {
				let mut p = current_exe().unwrap();
				p.pop();
				p.join(prefix.path)
			} else {
				Path::new(prefix.path).to_path_buf()
			};
			if prefix.append_app_id {
				new_path.push(app_id);
			}
			new_path.push(path);
			if new_path.exists() {
				return new_path;
			}
		}
	}

	path.to_path_buf()
}
