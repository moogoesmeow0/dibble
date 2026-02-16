use anyhow::Result;
use clap::{ArgAction, Parser};
use color_print::cprintln;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::{
    collections::hash_map::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(name = "dibble")]
#[command(version = "1.2")]
#[command(about = "Quick and local word definitions", long_about = None)]
struct Cli {
    /// The word to define
    word: String,

    /// Don't show example sentences
    #[arg(action = ArgAction::SetTrue, long, short)]
    no_examples: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if !&cli.word.chars().all(|c| c.is_alphabetic()) {
        cprintln!("<red>Invalid input: Word must contain only alphabetic characters.</red>");
        std::process::exit(1);
    }

    let word = cli.word.to_lowercase();
    let mut chars = word.chars();
    let first = chars.next().unwrap();

    let target: PathBuf = if let Some(second) = chars.next() {
        let mut path = PathBuf::from(String::from(first));
        path.push(format!("{}{}", String::from(first), String::from(second)));
        path
    } else {
        let mut path = PathBuf::from(String::from(first));
        path.push(String::from(first));
        path.into()
    };

    let contents = read_data(target.into())?;

    let data: DictionaryFile = from_str(&contents)?;

    if let Some(f) = data.get(&cli.word) {
        f.print_colored(!cli.no_examples);
    } else {
        cprintln!("<red>Word not found: {}</red>", cli.word);
    }

    Ok(())
}

fn read_data(path: PathBuf) -> Result<String> {
    let dirs: ProjectDirs = ProjectDirs::from("com.taranathan.dibble", "taran", "dibble").unwrap();

    let mut user_target = dirs.data_dir().to_path_buf();
    user_target.push(Path::new("dict"));
    user_target.push(&path);
    user_target.set_extension("json");

    let mut system_target = PathBuf::from("/usr/share/dibble/dict");
    system_target.push(&path);
    system_target.set_extension("json");

    let mut local = PathBuf::from("./dict");
    local.push(&path);
    local.set_extension("json");

    if let Ok(mut file) = File::open(&local) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        return Ok(contents);
    }

    if let Ok(mut file) = File::open(&user_target) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        return Ok(contents);
    }

    // system installation fallback
    if let Ok(mut file) = File::open(&system_target) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        return Ok(contents);
    }

    anyhow::bail!(
        "Dictionary file not found. Searched:\n  - {}\n  - {}",
        user_target.display(),
        system_target.display()
    )
}

pub type DictionaryFile = HashMap<String, Definition>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    /// The word being defined
    pub word: String,
    /// Array of Etymology objects, representing different meanings or origins of the word
    pub etymologies: Vec<Etymology>,
}

/// Represents a particular etymology or origin of a word
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Etymology {
    /// Array of Part of Speech objects within this etymology
    #[serde(rename = "partsOfSpeech")]
    pub parts_of_speech: Vec<PartOfSpeech>,
}

/// Represents a specific part of speech for a word
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartOfSpeech {
    /// The part of speech (e.g., "Noun", "Verb", "Adjective")
    #[serde(rename = "partOfSpeech")]
    pub part_of_speech: String,
    /// Array of Sense objects representing different meanings
    pub senses: Vec<Sense>,
}

/// Represents a specific sense or meaning of a word
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sense {
    /// The specific sense or meaning
    pub sense: String,
    /// Optional: Time period or usage context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(default)]
    pub examples: Vec<String>,
}

impl Definition {
    pub fn print_colored(&self, examples: bool) {
        //header
        cprintln!("<bold><cyan>{}</cyan></bold>", self.word);

        for (etym_idx, etymology) in self.etymologies.iter().enumerate() {
            if self.etymologies.len() > 1 {
                cprintln!("<bold><yellow>Etymology {}:</yellow></bold>", etym_idx + 1);
            }

            for pos in &etymology.parts_of_speech {
                cprintln!("  <bold><green>{}</green></bold>", pos.part_of_speech);

                for (sense_idx, sense) in pos.senses.iter().enumerate() {
                    cprintln!("    <bold>{}.</bold> {}", sense_idx + 1, sense.sense);

                    if let Some(date) = &sense.date {
                        if date.len() > 0 {
                            cprintln!("       <italic><dim>[{}]</dim></italic>", date);
                        }
                    }

                    if examples {
                        for example in &sense.examples {
                            cprintln!("       <dim>\"{}\"</dim>", example);
                        }
                    }
                }
                cprintln!();
            }
        }
    }
}
