use clap::Parser;
use litchi::{ odf, Document };
use std::path::{ Path, PathBuf };
use walkdir::{ WalkDir, DirEntry };

/// Simple program to search for (regex) patterns in odt and doc(x) files
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Pattern to search the files for
    pattern: String,
    
    /// Path to directory to search
    #[arg(long, default_value = ".")]
    path: PathBuf,
    
    /// How deep to recursively search[>= 1]
    #[arg(short, long, default_value_t = 1)]
    max_depth: usize,
    
    /// How verbose the response is [1-3]
    #[arg(short, long, default_value_t = 1,
        long_help = "How verbose the response is\n\
        1: Reports paths to files containing pattern\n\
        2: Reports paths and paragraph numbers containing pattern\n\
        3: Reports paths, paragraph numbers, and paragraph content containing pattern")]
    verbosity: i8,
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
fn get_paragraphs<'a>(content: &'a str, pattern: &str) -> Vec<(usize, &'a str)> {
    content
        .split("\n")
        .enumerate()
        .filter(|(_, s)| s.contains(pattern))
        .collect()
}

fn build_response(path: &Path, content: &str, pattern: &str, verbosity: &i8) -> String {
    let res = format!("{}", path.display());
    match verbosity {
        1 => res,
        2 => {
            let line_nums: Vec<String> = get_paragraphs(content, pattern)
                .iter()
                .map(|(i, _)| i.to_string())
                .collect();
            format!("{} ({})", res, line_nums.join(", "))
        },
        _ => {
            let lines_str: String = get_paragraphs(content, pattern)
                .iter()
                .map(|(i, s)| format!("  {}: {}", i, s))
                .collect::<Vec<String>>()
                .join("\n");
            format!("{}\n{}", res, lines_str)
        },
    }
}
fn search_file(path: &Path, pattern: &str, verbosity: &i8) -> Result<String, SearchError> {
    let content = match path.extension().and_then(|e| e.to_str()) {
        Some("odt") => {
            let mut doc = odf::Document::open(path)?;
            doc.text()?
        }
        Some("docx") | Some("doc") => {
            let doc = Document::open(path)?;
            doc.text()?
        }
        _ => { String::from("") }
    };
    if content.contains(pattern) {
        Ok(build_response(path, &content, pattern, verbosity))
    } else{
        Err(SearchError(format!("{} does not cotain {}", path.display(), pattern)))
    }
}
pub fn main() {
    let args = Args::parse();
    WalkDir::new(args.path)
        .max_depth(args.max_depth)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| is_valid(&entry))
        .for_each(|entry| {
            match search_file(entry.path(), &args.pattern, &args.verbosity) {
                Ok(v) => {
                    println!("{}", v);
                }
                Err(_) => {}
            }
        });
}
