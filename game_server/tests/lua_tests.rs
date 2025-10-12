use libtest_mimic::{Arguments, Failed, Trial};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct Test {
    pub script_asset_path: PathBuf,
}

/// Discover all test files in the data/scripts/tests directory
pub fn discover_all_tests(manifest_dir: PathBuf, filter: impl Fn(&Test) -> bool) -> Vec<Test> {
    let tests_root = manifest_dir.join("data").join("scripts").join("tests");
    let mut test_files = Vec::new();

    if !tests_root.exists() {
        eprintln!("Tests directory not found: {}", tests_root.display());
        return test_files;
    }

    for entry in WalkDir::new(&tests_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("lua") {
            // Get the relative path from the tests directory
            let relative = path.strip_prefix(&tests_root).unwrap_or(path);

            let test = Test {
                script_asset_path: relative.to_path_buf(),
            };

            if filter(&test) {
                test_files.push(test);
            }
        }
    }

    test_files
}

trait TestExecutor {
    fn execute(self) -> Result<(), Failed>;
}

impl TestExecutor for Test {
    fn execute(self) -> Result<(), Failed> {
        let script_asset_path = self.script_asset_path;

        match execute_lua_test(&script_asset_path) {
            Ok(success) => {
                if success {
                    Ok(())
                } else {
                    Err(Failed::from("Test script failed"))
                }
            }
            Err(e) => Err(Failed::from(format!("Test execution error: {e:?}"))),
        }
    }
}

fn execute_lua_test(script_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let full_script_path = manifest_dir
        .join("data")
        .join("scripts")
        .join("tests")
        .join(script_path);

    if !full_script_path.exists() {
        return Err(format!("Test script not found: {}", full_script_path.display()).into());
    }

    let script_content = std::fs::read_to_string(&full_script_path)?;

    // Create a Lua context
    let lua = mlua::Lua::new();
    let base_path = manifest_dir.join("data").join("scripts");
    let package: mlua::Table = lua.globals().get("package")?;
    let current_path: String = package.get("path")?;
    let custom_path = format!(
        "{}{}{};{}",
        base_path.display(),
        std::path::MAIN_SEPARATOR,
        "?.lua",
        current_path
    );
    package.set("path", custom_path)?;

    // Add a simple print function
    let print_fn = lua.create_function(|_, msg: String| {
        println!("{}", msg);
        Ok(())
    })?;
    lua.globals().set("print", print_fn)?;

    // Execute the script directly without any content preprocessing
    let result: mlua::Value = lua.load(&script_content).eval()?;
    match result {
        mlua::Value::Boolean(success) => Ok(success),
        mlua::Value::Table(table) => {
            if let Ok(success) = table.get::<bool>("success") {
                Ok(success)
            } else if let Ok(stats) = table.get::<mlua::Table>("stats") {
                let failed = stats.get::<i32>("failed").unwrap_or(0);
                Ok(failed == 0)
            } else {
                Ok(false)
            }
        }
        mlua::Value::Nil => Ok(false),
        _ => Ok(false),
    }
}

fn main() {
    let mut args = Arguments::from_args();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    args.test_threads = Some(1); // Force single-threaded to avoid issues with Lua state

    let tests = discover_all_tests(manifest_dir.clone(), |test| {
        // Check if the file is a test file (contains "test_" or is named "test_runner.lua")
        let path_str = test.script_asset_path.to_string_lossy();
        let file_name = test
            .script_asset_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        path_str.contains("test_") || file_name == "test_runner.lua"
    })
    .into_iter()
    .enumerate()
    .map(|(i, t)| {
        let test_name = format!(
            "lua_tests_{:02}_{}",
            i,
            t.script_asset_path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
        );
        Trial::test(test_name, move || t.execute())
    })
    .collect::<Vec<_>>();

    if tests.is_empty() {
        eprintln!("No test files found. Make sure test files exist in data/scripts/tests/");
        std::process::exit(1);
    }
    libtest_mimic::run(&args, tests).exit();
}
