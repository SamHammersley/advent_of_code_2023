extern crate const_format;
extern crate cargo;

use std::env;
use std::fs;

use const_format::concatcp;

use cargo::Config;
use cargo::ops::run;
use cargo::core::Workspace;
use cargo::util::command_prelude::CompileMode;
use cargo::ops::CompileOptions;
use cargo::ops::Packages;

const SESSION_ID_VAR_KEY: &str = "AOC_SESSION_ID";
const USER_AGENT_VAR_KEY: &str = "AOC_USER_AGENT";
const ADVENT_OF_CODE_URL: &str = "https://adventofcode.com/2023/day/";
const SOLUTIONS_DIR_PATH: &str = "solutions/";
const INPUT_DIR_PATH: &str = concatcp!(SOLUTIONS_DIR_PATH, "/input/");

pub fn get_advent_of_code_input(current_working_directory: &str, day: u32) -> Result<String, anyhow::Error> {
    let input_path = &format!("{INPUT_DIR_PATH}day_{day}.txt");
    
    fs::read_to_string(input_path).or_else(|_| -> Result<String, anyhow::Error>
    {
        let session_id = &env::var(SESSION_ID_VAR_KEY)?;
        let aoc_user_agent = &env::var(USER_AGENT_VAR_KEY)?;
        let url: &str = &format!("{ADVENT_OF_CODE_URL}{day}/input");

        let request = ureq::get(url)
            .set("Cookie", &format!("session={session_id}"))
            .set("User-Agent", aoc_user_agent);

        let challenge_input = request
            .call()?
            .into_string()?;

        // persist the input to disk
        fs::write(input_path, &challenge_input)?;

        Ok(challenge_input.clone())
    })
}

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 2, "Day number is required as argument!");

    let day_number = args[1].parse::<u32>()?;
    let input = get_advent_of_code_input(day_number)?;

    let solution_dir = &format!("{SOLUTIONS_DIR_PATH}day_{day_number}/");

    let config = Config::default()?;

    let mut manifest_path = env::current_dir()?;
    manifest_path.push(solution_dir);
    manifest_path.push("Cargo");
    manifest_path.set_extension("toml");

    let workspace = Workspace::new(manifest_path.as_path(), &config)?;
    let mut compile_options = CompileOptions::new(&config, CompileMode::Build)?;
    // For simplicity's sake, just compile all packages. Doesn't work without
    // specifying this.
    compile_options.spec = Packages::All;

    run(&workspace, &compile_options, &[input.into()])
}
