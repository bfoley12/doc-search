use clap::Parser;
use litchi::{ odf, Document };
use std::path::{ Path, PathBuf };
use walkdir::{ WalkDir, DirEntry };

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    pattern: String,
    
    #[arg(long, default_value = ".")]
    path: PathBuf,
    
    #[arg(short, long, default_value_t = 1)]
    recursive: i8,
    
    #[arg(short, long, default_value_t = 1)]
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
    let args = Cli::parse();
    let mut wd = WalkDir::new(args.path);
    if args.recursive < 1 {
        wd = wd.max_depth(1);
    }
    wd.into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| is_valid(&entry))
        .for_each(|entry| {
            match search_file(entry.path(), &args.pattern, &args.verbosity) {
                Ok(v) => {
                    println!("{}", v);
                }
                Err(_) => {
                    
                }
            }
        });
}
