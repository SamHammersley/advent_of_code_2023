extern crate cargo;
extern crate const_format;
extern crate regex;

use std::env;
use std::fs;
use std::path::PathBuf;

use const_format::concatcp;

use cargo::core::Workspace;
use cargo::ops::CompileOptions;
use cargo::ops::Packages;
use cargo::ops::*;
use cargo::util::command_prelude::CompileMode;
use cargo::Config;
use regex::Regex;

const SESSION_ID_VAR_KEY: &str = "AOC_SESSION_ID";
const USER_AGENT_VAR_KEY: &str = "AOC_USER_AGENT";
const ADVENT_OF_CODE_URL: &str = "https://adventofcode.com/2023/day/";
const SOLUTIONS_DIR_PATH: &str = "solutions/";
const INPUT_DIR_PATH: &str = concatcp!(SOLUTIONS_DIR_PATH, "/input/");

pub fn get_advent_of_code_input(mut cwd_path: PathBuf, day: u32) -> Result<String, anyhow::Error> {
    let input_file_name = &format!("day_{day}.txt");
    cwd_path.push(input_file_name);

    fs::read_to_string(cwd_path.as_path()).or_else(|_| -> Result<String, anyhow::Error> {
        let session_id = &env::var(SESSION_ID_VAR_KEY)?;
        let aoc_user_agent = &env::var(USER_AGENT_VAR_KEY)?;
        let url: &str = &format!("{ADVENT_OF_CODE_URL}{day}/input");

        let request = ureq::get(url)
            .set("Cookie", &format!("session={session_id}"))
            .set("User-Agent", aoc_user_agent);

        let challenge_input = request.call()?.into_string()?;

        // persist the input to disk
        fs::write(cwd_path.as_path(), &challenge_input)?;

        Ok(challenge_input.clone())
    })
}

/// Searches for directories in the given directory whose names match the regex pattern:
///     `day_(\d+)`
///
/// # Arguments
///
/// * `cwd_path` - A PathBuf to the directory to look in.
///
/// This doesn't verify anything about the directory, we currently just assume that it's a
/// directory that contains a binary crate.
fn find_last_day(cwd_path: PathBuf) -> Result<u32, anyhow::Error> {
    let re = Regex::new(r"day_(\d+)").unwrap();

    let last_entry = fs::read_dir(cwd_path)?
        .filter_map(|entry_result| {
            let entry = entry_result.ok()?;
            if entry.file_type().ok()?.is_dir() {
                let file_name = entry.file_name();
                let file_name = file_name.to_str()?;
                if let Some(target_match) = re.captures(file_name) {
                    if let Ok(parsed_value) = target_match[1].parse::<u32>() {
                        return Some(parsed_value);
                    }
                }
            }
            None
        })
        .last();
    last_entry.ok_or(anyhow::anyhow!("Failed to find any solutions!"))
}

fn main() -> Result<(), anyhow::Error> {
    let cwd_path = env::current_dir()?;
    let args: Vec<String> = env::args().collect();

    // if there is an argument for the day to run, use that or find last day_x crate
    let day_number = match args.len() {
        x if x != 2 => find_last_day(cwd_path.join(SOLUTIONS_DIR_PATH))?,
        _ => args[1].parse::<u32>()?,
    };

    let day_directory = &format!("day_{day_number}/");

    let mut manifest_path = cwd_path.join(SOLUTIONS_DIR_PATH);
    manifest_path.push(day_directory);
    manifest_path.push("Cargo");
    manifest_path.set_extension("toml");

    let config = Config::default()?;

    let workspace = Workspace::new(manifest_path.as_path(), &config)?;
    let mut compile_options = CompileOptions::new(&config, CompileMode::Build)?;
    // For simplicity's sake, just compile all packages. Doesn't work without
    // specifying this.
    compile_options.spec = Packages::All;

    let input = get_advent_of_code_input(cwd_path.join(INPUT_DIR_PATH), day_number)?;
    run(&workspace, &compile_options, &[input.into()])
}
