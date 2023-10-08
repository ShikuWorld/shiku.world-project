use convert_case::{Case, Casing};
use std::fs;
use std::path::{Path, PathBuf};
use std::{env, io};

fn main() {
    let modules_to_copy = [
        "Slime1Module",
        "LobbyModule",
        "LoginModule",
        "ArgumentModule",
    ];
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut out_dir_path = PathBuf::from(Path::new(out_dir.as_str()));
    out_dir_path.pop();
    out_dir_path.pop();
    out_dir_path.pop();
    out_dir_path = out_dir_path.join("out");

    for module_to_copy in modules_to_copy {
        let module_name_snake_case = module_to_copy.to_string().to_case(Case::Snake);
        let path = format!("src/{}/resources", module_name_snake_case);
        let path_to_module_resources = Path::new(path.as_str());

        copy_dir_all(
            path_to_module_resources.join("shared"),
            out_dir_path.join("shared").join(module_to_copy),
        )
        .unwrap();

        copy_dir_all(
            path_to_module_resources,
            out_dir_path.join(module_name_snake_case).join("resources"),
        )
        .unwrap();
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
