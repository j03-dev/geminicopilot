use std::fs;
use std::io::Read;
use std::path::Path;
use std::{error::Error, process::exit};

use dotenv::dotenv;
use serde::{Deserialize, Serialize};

const URL: &str =
    "https://generativelanguage.googleapis.com/v1/models/gemini-pro:generateContent?key=";

#[derive(Serialize, Deserialize, Clone)]
struct Part {
    text: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Candidate {
    content: Content,
}

#[derive(Serialize, Deserialize, Clone)]
struct Response {
    candidates: Vec<Candidate>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Body {
    contents: Vec<Content>,
}

const PROMPT: &str = r#"
You are developing an AI-powered coding assistant called "GeminiCopilot" to enhance developers' coding experience. GeminiCopilot aims to provide intelligent suggestions, automate repetitive tasks, and support collaborative coding across various programming languages and frameworks.

Key Objectives:
1. **Intelligent Assistance:** Understand code context, provide suggestions, error detection, and optimization tips.
2. **Language Agnostic:** Support multiple languages and frameworks.
3. **Adaptability:** Learn coding style, adapt over time.
4. **Efficiency:** Automate repetitive tasks like code generation and refactoring.
5. **Documentation Assistance:** Provide API documentation and usage examples.
6. **Collaboration Support:** Facilitate collaborative coding, resolve merge conflicts.

Instructions:
- Design and implement GeminiCopilot's architecture for scalability and maintainability.
- Utilize NLP and machine learning for code understanding.
- Develop a robust testing framework for reliability.
- Prioritize UX with an intuitive interface and customization options.
- Iterate based on user feedback and industry trends.

**Deliverables:**
1. Detailed design document outlining architecture and technologies.
2. Source code with comprehensive documentation and installation instructions.
3. Test suite covering unit, integration, and end-to-end tests.
4. Deployment strategy for desktop, IDE plugins, and cloud services.

**Evaluation Criteria:**
- Accuracy and relevance of suggestions.
- Language and framework support.
- Adaptability and personalization.
- Efficiency in task automation.
- Documentation quality and UI design.
- System scalability and performance.
- Responsiveness to user feedback.
"#;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

async fn ask_gemini(text: String) -> Result<Response> {
    dotenv().ok();
    let api_key = std::env::var("API_KEY").expect("pls check your env file");
    let api_url = format!("{URL}{api_key}");
    let body = Body {
        contents: vec![Content {
            role: "user".to_owned(),
            parts: vec![Part {
                text: format!("{PROMPT}: {text}"),
            }],
        }],
    };
    let response = reqwest::Client::new()
        .post(api_url)
        .json(&body)
        .send()
        .await?;

    Ok(response.json().await?)
}

fn walk_directory(dir_path: &Path, all_content: &mut String, ignore_list: &[String]) -> Result<()> {
    if ignore_list
        .iter()
        .any(|pattern| dir_path.starts_with(pattern))
    {
        return Ok(());
    }

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_directory(&path, all_content, ignore_list)?;
        } else {
            if let Some(file_content) = read_file_content(&path)? {
                let dir_path_str = path
                    .parent()
                    .ok_or("Failed to get parent directory")?
                    .to_string_lossy();
                all_content.push_str(&format!("Directory: {}\n", dir_path_str));
                all_content.push_str(&file_content);
                all_content.push_str("\n\n");
            }
        }
    }
    Ok(())
}

// Function to read the content of a file
fn read_file_content(file_path: &Path) -> Result<Option<String>> {
    // Read the file's contents into a string
    let mut file = fs::File::open(file_path)?;
    let mut content = String::new();
    if let Err(err) = file.read_to_string(&mut content) {
        eprintln!("{err} => {path}", path = file_path.to_str().unwrap());
        exit(100);
    }

    Ok(Some(content))
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Check if there are enough arguments
    if args.len() < 2 {
        eprintln!("Usage: {} <input_text>", args[0]);
        std::process::exit(1);
    }

    // Extract user input text from arguments
    let user_input_text = &args[1];

    // Specify the directory you want to walk
    let current_dir = ".";
    let mut ignore_list = Vec::new();
    walk_gitignore(Path::new(current_dir), &mut ignore_list)?;

    let mut current_directory_content = String::new();

    // Recursively walk the directory
    walk_directory(
        Path::new(current_dir),
        &mut current_directory_content,
        &ignore_list,
    )?;

    let response = ask_gemini(format!(
        "{current_directory_content}\n:**and this is the directive**  => {user_input_text}"
    ))
    .await?;
    for part in response.candidates[0].content.parts.clone() {
        println!("{}", part.text);
    }
    Ok(())
}

fn walk_gitignore(dir_path: &Path, ignore_list: &mut Vec<String>) -> Result<()> {
    let gitignore_path = dir_path.join(".gitignore");
    if gitignore_path.exists() {
        let gitignore_content = fs::read_to_string(gitignore_path)?;
        ignore_list.extend(gitignore_content.lines().map(|s| format!("./{s}")));
    }

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_gitignore(&path, ignore_list)?;
        }
    }
    Ok(())
}
