use std::path::Path;
use std::fs::{read_dir, create_dir_all};
use wesl::Wesl;

fn main() {
    let shader_dir = "res/";
    let out_dir = "res/compiled/";

    if !Path::new(shader_dir).exists() {
        return;
    }

    create_dir_all(out_dir).unwrap_or_else(|e| panic!("Failed to create output dir: {e}"));

    let entries = read_dir(shader_dir).unwrap_or_else(|e| panic!("Failed to read shader directory: {e}"));

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "wesl") {
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                let package_path = format!("package::{}", file_stem);
                let module_path = package_path.parse().unwrap_or_else(|e| {
                    panic!("Failed to parse WESL path for {file_stem}: {e}");
                });
                let result = Wesl::new(shader_dir)
                    .compile(&module_path)
                    .unwrap_or_else(|e| panic!("Failed to compile {file_stem}.wesl: {e}"));
                let out_path = format!("{shader_dir}compiled/{file_stem}.wgsl");
                result.write_to_file(&out_path)
                    .unwrap_or_else(|e| panic!("Failed to write {out_path}: {e}"));
            }
        }
    }
}