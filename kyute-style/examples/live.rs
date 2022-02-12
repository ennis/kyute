use anyhow::Result;
use kyute_style::{Arena, Document};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::{fs::File, io::Read, path::Path, thread};

const TESTBENCH_PATH: &str = "kyute-style/tests/testbench.sty";

fn reparse() -> Result<()> {
    let mut contents = String::new();
    let mut file = File::open(TESTBENCH_PATH)?;
    file.read_to_string(&mut contents)?;
    // run the lexer
    eprintln!("--- Parser ---");
    let mut arena = Arena::new();
    let doc = Document::parse(&contents, &arena);
    match doc {
        Ok(m) => {
            eprintln!("--- DUMP: \n {:#?}", m);
            eprintln!("--- JSON: \n {}", m.to_json().to_string());
        }
        Err(e) => eprintln!("{:?}", e),
    }
    Ok(())
}

fn main() -> Result<()> {
    let mut watcher = notify::recommended_watcher(|res| {
        match res {
            Ok(event) => {
                //println!("watch event: {:?}", event);
                if let Err(e) = reparse() {
                    println!("parse error: {}", e)
                }
            }
            Err(e) => println!("watch error: {}", e),
        }
    })?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(Path::new(TESTBENCH_PATH), RecursiveMode::NonRecursive)?;

    thread::park();
    Ok(())
}
