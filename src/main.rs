use clap::Parser;
use litchi::{ odf, Document };
use std::path::{ Path, PathBuf };
use walkdir::{ WalkDir, DirEntry };
use regex::{Regex, RegexBuilder};

/// Simple program to search for (regex) patterns in odt and doc(x) files
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Pattern to search the files for
    pattern: String,
    
    /// Path to directory to search
    #[arg(long, default_value = ".")]
    path: PathBuf,
    
    /// How deep to recursively search[>=0]
    #[arg(short, long, default_value_t = usize::MAX)]
    depth: usize,
    
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
fn get_paragraphs<'a>(content: &'a str, pattern: &Regex) -> Vec<(usize, &'a str)> {
    content
        .split("\n")
        .enumerate()
        .filter(|(_, s)| pattern.is_match(s))
        .collect()
}

fn build_response(path: &Path, content: &str, pattern: &Regex, verbosity: &i8) -> String {
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
fn search_file(path: &Path, pattern: &Regex, verbosity: &i8) -> Result<String, SearchError> {
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
    if pattern.is_match(&content) {
        Ok(build_response(path, &content, pattern, verbosity))
    } else{
        Err(SearchError(format!("{} does not cotain {}", path.display(), pattern)))
    }
}
pub fn main() {
    let args = Args::parse();
    let pattern = RegexBuilder::new(&args.pattern)
        .multi_line(true)
        .build()
        .unwrap_or_else(|e| {
        eprintln!("Invalid pattern: {}", e);
        std::process::exit(1);
    });
    WalkDir::new(args.path)
        .max_depth(args.depth)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| is_valid(&entry))
        .for_each(|entry| {
            match search_file(entry.path(), &pattern, &args.verbosity) {
                Ok(v) => {
                    println!("{}", v);
                }
                Err(_) => {}
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();

        fs::write(dir.path().join("ignore.txt"), "ranomd content").unwrap();

        fs::write(dir.path().join("ignore.pdf"), "random content").unwrap();

        let fixture_dir = Path::new("test_data");
        fs::copy(fixture_dir.join("sample.odt"), dir.path().join("sample.odt")).unwrap();
        fs::copy(fixture_dir.join("sample.docx"), dir.path().join("sample.docx")).unwrap();
        fs::copy(fixture_dir.join("sample.doc"), dir.path().join("sample.doc")).unwrap();

        dir
    }

    #[test]
    fn test_ignores_txt_files() {
        let dir = setup_test_dir();
        let entry = WalkDir::new(dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .find(|e| e.path().extension().map_or(false, |ext| ext == "txt"))
            .unwrap();
        assert!(!is_valid(&entry));
    }

    #[test]
    fn test_accepts_odt_files() {
        let dir = setup_test_dir();
        let entry = WalkDir::new(dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .find(|e| e.path().extension().map_or(false, |ext| ext == "odt"))
            .unwrap();
        assert!(is_valid(&entry));
    }

    #[test]
    fn test_search_finds_pattern() {
        let dir = setup_test_dir();
        let entry = WalkDir::new(dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .find(|e| e.path().extension().map_or(false, |ext| ext == "odt"))
            .unwrap();
        let reg = Regex::new("hello world").unwrap();
        assert!(search_file(&entry.path(), &reg, &1).is_ok());
    }

    #[test]
    fn test_search_rejects_missing_pattern() {
        let dir = setup_test_dir();
        let entry = WalkDir::new(dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .find(|e| e.path().extension().map_or(false, |ext| ext == "odt"))
            .unwrap();
        let reg = Regex::new("nonexistent gibberish").unwrap();
        assert!(!search_file(&entry.path(), &reg, &1).is_ok());
    }
    
    #[test]
    fn test_get_paragraphs() {
        let content = "the cat sat\na dog ran\nthe cat returned";
        let reg = Regex::new("cat").unwrap();
        let result = get_paragraphs(content, &reg);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 0);
        assert_eq!(result[1].0, 2);
    }
    
    #[test]
    fn test_build_response_verbosity_1() {
        let path = Path::new("/test/file.odt");
        let content = "the cat sat\na dog ran";
        let reg = Regex::new("cat").unwrap();
        let result = build_response(path, content, &reg, &1);
        assert_eq!(result, "/test/file.odt");
    }
    
    #[test]
    fn test_build_response_verbosity_2() {
        let path = Path::new("/test/file.odt");
        let content = "the cat sat\na dog ran\nthe cat returned";
        let reg = Regex::new("cat").unwrap();
        let result = build_response(path, content, &reg, &2);
        assert_eq!(result, "/test/file.odt (0, 2)");
    }
    
    #[test]
    fn test_regex_character_class() {
        let content = "the cat sat\na dog ran\nthe bat returned";
        let reg = Regex::new("[cb]at").unwrap();
        let result = get_paragraphs(content, &reg);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, "the cat sat");
        assert_eq!(result[1].1, "the bat returned");
    }
    
    #[test]
    fn test_regex_wildcard() {
        let content = "hello world\nhello there\ngoodbye world";
        let reg = Regex::new("hello.*world").unwrap();
        let result = get_paragraphs(content, &reg);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, 0);
    }
    
    #[test]
    fn test_regex_start_of_line() {
        let content = "cat is here\nthe cat sat\ncat returns";
        let reg = RegexBuilder::new("^cat").multi_line(true).build().unwrap();
        let result = get_paragraphs(content, &reg);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 0);
        assert_eq!(result[1].0, 2);
    }
    
    #[test]
    fn test_regex_end_of_line() {
        let content = "I saw a cat\nthe dog ran\nhere is a cat";
        let reg = RegexBuilder::new("cat$").multi_line(true).build().unwrap();
        let result = get_paragraphs(content, &reg);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 0);
        assert_eq!(result[1].0, 2);
    }
    
    #[test]
    fn test_regex_case_insensitive() {
        let content = "Hello World\nhello world\nHELLO WORLD";
        let reg = RegexBuilder::new("hello world").case_insensitive(true).build().unwrap();
        let result = get_paragraphs(content, &reg);
        assert_eq!(result.len(), 3);
    }
    
    #[test]
    fn test_regex_digit_matching() {
        let content = "item 1\nno number here\nitem 42\nstill nothing";
        let reg = Regex::new(r"\d+").unwrap();
        let result = get_paragraphs(content, &reg);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 0);
        assert_eq!(result[1].0, 2);
    }
    
    #[test]
    fn test_plain_string_still_works() {
        let content = "hello world\ngoodbye world\nhello again";
        let reg = Regex::new("hello").unwrap();
        let result = get_paragraphs(content, &reg);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 0);
        assert_eq!(result[1].0, 2);
    }
    
    #[test]
    fn test_regex_no_match() {
        let content = "hello world\ngoodbye world";
        let reg = Regex::new(r"^\d{3}-\d{4}$").unwrap();
        let result = get_paragraphs(content, &reg);
        assert_eq!(result.len(), 0);
    }
}
