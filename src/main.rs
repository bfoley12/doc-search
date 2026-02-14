use clap::Parser;
use litchi::odf;
use std::path::PathBuf;
use walkdir::{ WalkDir, DirEntry };

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    pattern: String,
    
    #[arg(long, default_value = ".")]
    path: PathBuf,
    
    #[arg(short, long, default_value_t = 1)]
    recursive: i8,
}
fn is_valid(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.ends_with(".odt") | s.ends_with(".doc") | s.ends_with(".docx"))
        .unwrap_or(false)
}
#[derive(Debug)]
struct SearchError(String);
impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for SearchError {}
impl From<litchi::Error> for SearchError {
    fn from(e: litchi::Error) -> Self {
        SearchError(e.to_string())
    }
}
fn search_file(entry: &DirEntry, pattern: &str) -> Result<String, SearchError> {
    let mut doc = odf::Document::open(entry.path())?;
    let content = doc.text()?;
    if content.contains(pattern) {
        Ok(format!("{}", entry.path().display()))
    } else {
        Err(SearchError(format!("{} does not contain {}", entry.path().display(), pattern)))
    }
}
pub fn main() {
    let args = Cli::parse();
    WalkDir::new(args.path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| is_valid(&entry))
        .for_each(|entry| {
            match search_file(&entry, &args.pattern) {
                Ok(v) => {
                    println!("{}", v)
                }
                Err(_) => {
                    
                }
            }
        });
}
