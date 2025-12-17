//!
//! solx test updater executable.
//!

pub(crate) mod arguments;

use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::Write;

use clap::Parser;
use colored::Colorize;

use self::arguments::Arguments;

///
/// solx test updater executable.
///
fn main() {
    let arguments = Arguments::parse();

    let file = OpenOptions::new()
        .read(true)
        .open(arguments.index.clone())
        .expect("Failed to open file");
    let reader = BufReader::new(file);
    let old_index: solx_solc_test_adapter::FSEntity =
        serde_yaml::from_reader(reader).expect("Failed to read index");

    let mut new_index =
        solx_solc_test_adapter::FSEntity::index(&arguments.source).expect("Failed to update index");
    let changes = old_index
        .update(&mut new_index, arguments.source.as_path())
        .expect("Failed to update tests");

    println!("{} files created:\n", changes.created.len());
    for file in changes.created {
        println!("{}", file.to_string_lossy().green());
    }
    println!();

    println!("{} files deleted:\n", changes.deleted.len());
    for file in changes.deleted {
        println!("{}", file.to_string_lossy().red());
    }
    println!();

    println!("{} files updated:\n", changes.updated.len());
    for file in changes.updated {
        println!("{}", file.to_string_lossy());
    }
    println!();

    println!(
        "{} conflicts(both modified, were overwritten):\n",
        changes.conflicts.len()
    );
    for file in changes.conflicts {
        println!("{}", file.to_string_lossy().bright_red());
    }
    println!();

    let new_index = serde_yaml::to_string(&new_index).expect("Serialization");
    let mut file_to_write = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(arguments.index)
        .expect("Failed to open file");
    file_to_write
        .write_all(new_index.as_bytes())
        .expect("Failed to write to the output file");

    println!("Test files successfully updated");
}
