extern crate liquid;
extern crate pulldown_cmark;

use pulldown_cmark::HeadingLevel;
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::Command;
use std::{fs, io};
use toml;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Welcome to the blog generator!");
        println!("Usage: terb COMMAND");
        println!("-I Init Blog");
        println!("-G Generate Blog");
        println!("-S Run Web Server");
        println!("-P Push to Git repository");
        println!("Version 0.0.2");
        return;
    }
    let command = &args[1];
    match command.as_ref() {
        "-G" => {
            check_dir("./out");
            check_dir("./out/posts");
            check_dir("./posts");
            check_dir(".terb");
            generate_posts();
            generate_list();
            println!("Success!")
        }
        "-S" => {
            println!("Serving HTTP on 127.0.0.1 port 7878 (http://localhost:7878/)");
            println!("Press Ctrl+C to exit.");
            let file_path = Path::new("out/404.html");
            if !file_path.exists() {
                let mut file = match File::create(file_path) {
                    Ok(file) => file,
                    Err(err) => panic!("Error creating file: {}", err),
                };
                file.write_all(b"<h1>404 - Page Not Found</h1>").unwrap();
            }
            let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
            for stream in listener.incoming() {
                let stream = stream.unwrap();

                handle_connection(stream);
            }
        }
        "-I" => {
            let dirs = ["posts", "out", "out/posts", ".terb", ".terb/template"];
            for dir in &dirs {
                match fs::create_dir_all(dir) {
                    Ok(_) => println!("{} directory created", dir),
                    Err(e) => println!("Error creating {} directory: {:?}", dir, e),
                }
            }
            let mut config = toml::value::Table::new();
            let keys = [
                "blogtitle",
                "description",
                "author",
                "list_path",
                "git_repo",
            ];
            for key in &keys {
                let mut config_string = String::new();
                println!("Enter {}:", key);
                io::stdin().read_line(&mut config_string).unwrap();
                let value = config_string.trim();
                if key == &"git_repo" {
                    let repos: Vec<String> =
                        value.split(",").map(|s| s.trim().to_string()).collect();
                    config.insert(
                        key.to_string(),
                        toml::Value::Array(
                            repos.into_iter().map(|s| toml::Value::String(s)).collect(),
                        ),
                    );
                } else {
                    config.insert(key.to_string(), toml::Value::String(value.to_string()));
                }
            }
            let config_string = toml::to_string(&toml::Value::Table(config)).unwrap();
            let mut file = File::create(".terb/config.toml").unwrap();
            file.write_all(config_string.as_bytes()).unwrap();
            println!("Do you want to initialize out directory as a git project? (Y/n) Press enter to confirm");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            if input == "Y" || input == "" {
                Command::new("git")
                    .arg("init")
                    .current_dir("out")
                    .output()
                    .expect("Failed to initialize git repository.");
                Command::new("git")
                    .arg("checkout")
                    .arg("-b")
                    .arg("main")
                    .current_dir("out")
                    .output()
                    .expect("Failed to create branch main and switch to it.");
            }

            println!("Finish!");
        }
        "-P" => {
            let config: toml::Value =
                toml::from_str(fs::read_to_string(".terb/config.toml").unwrap().as_str()).unwrap();
            let git_repo = config.get("git_repo").unwrap().as_array().unwrap();

            for repo in git_repo {
                let repo_url = repo.as_str().unwrap();

                let output = Command::new("git")
                    .current_dir("out")
                    .args(&["remote", "add", "origin", repo_url])
                    .output()
                    .expect("Failed to add remote repository.");

                if output.status.success() {
                    Command::new("git")
                        .current_dir("out")
                        .args(&["add", "."])
                        .output()
                        .expect("Failed to add files.");

                    let commit_output = Command::new("git")
                        .current_dir("out")
                        .args(&["commit", "-m", "Initial commit"])
                        .output()
                        .expect("Failed to commit files.");

                    if commit_output.status.success() {
                        Command::new("git")
                            .current_dir("out")
                            .args(&["push", "-u", "origin", "main", "--force"])
                            .output()
                            .expect("Failed to push to remote repository.");
                    } else {
                        println!("Nothing to commit.");
                    }
                }
            }
        }
        _ => println!("Invalid command"),
    }
}
fn check_dir(path: &str) {
    let path = Path::new(path);
    if !path.exists() {
        println!(
            "Directory '{}' not found, create now? (Y/n)",
            path.display()
        );
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim() == "Y" {
            fs::create_dir(path).expect("Error creating directory");
        } else {
            println!("Please mkdir first");
            std::process::exit(1);
        }
    }
}
fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    let get = b"GET /";
    let (status_line, filename) = if buffer.starts_with(get) {
        let request_str = String::from_utf8_lossy(&buffer);
        let mut request_lines = request_str.split("\r\n");
        let request_line = request_lines.next().unwrap();
        let mut request_parts = request_line.split(" ");
        let _ = request_parts.next().unwrap();
        let requested_path = request_parts.next().unwrap();
        if requested_path == "/" {
            ("HTTP/1.1 200 OK\r\n\r\n", "out/index.html".to_string())
        } else {
            let requested_path = requested_path[1..].to_string();
            let file_path = Path::new("out").join(requested_path);
            if file_path.exists() {
                (
                    "HTTP/1.1 200 OK\r\n\r\n",
                    file_path.to_str().unwrap().to_string(),
                )
            } else {
                ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "out/404.html".to_string())
            }
        }
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "out/404.html".to_string())
    };
    let contents = fs::read_to_string(filename).unwrap();
    let response = format!("{}{}", status_line, contents);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
fn extract_title(file_path: &Path) -> String {
    let mut file = fs::File::open(file_path).expect("Error opening file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Error reading file");
    let parser = Parser::new(&content);
    let mut title = String::new();
    for event in parser {
        match event {
            Event::Start(Tag::Heading(HeadingLevel::H1, _, _)) => {
                title = String::new();
            }
            Event::Text(text) => {
                if title.is_empty() {
                    title.push_str(&text);
                    break;
                }
            }
            _ => {}
        }
    }
    if title.is_empty() {
        "Untitled".to_owned()
    } else {
        title
    }
}
fn extract_date(file_path: &Path) -> String {
    let mut file = fs::File::open(file_path).expect("Error opening file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Error reading file");
    let parser = Parser::new(&content);
    let mut date = String::new();
    let mut h2_count = 0;
    for event in parser {
        match event {
            Event::Start(Tag::Heading(HeadingLevel::H2, _, _)) => {
                h2_count += 1;
                if h2_count == 1 {
                    date = String::new();
                }
            }
            Event::Text(text) => {
                if h2_count == 1 {
                    date.push_str(&text);
                    break;
                }
            }
            _ => {}
        }
    }
    if date.is_empty() {
        "Untitled".to_owned()
    } else {
        date
    }
}
fn generate_html(md_file: &Path) -> String {
    let md_string = fs::read_to_string(md_file).expect("Error reading Markdown file");
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let parser = Parser::new_ext(&md_string, options);
    let mut html_string = String::new();
    html::push_html(&mut html_string, parser);
    html_string
}
fn generate_posts() {
    let posts_dir = Path::new("posts");
    let liquid = liquid::ParserBuilder::with_stdlib().build().unwrap();
    let template = liquid
        .parse(
            &fs::read_to_string(".terb/template/post.liquid").expect("Error reading template file"),
        )
        .expect("Error parsing template file");
    for entry in posts_dir.read_dir().expect("Error reading posts directory") {
        let md_file = entry.expect("Error reading entry").path();
        let title = extract_title(&md_file);
        let date = extract_date(&md_file);
        let html_string = generate_html(&md_file);
        let file_name = md_file
            .file_stem()
            .expect("Error getting file stem")
            .to_str()
            .expect("Error converting file stem to string");
        let data = liquid::object!({
            "title": title,
            "date": date,
            "content": html_string
        });
        let html_string = template.render(&data).expect("Error rendering template");
        let html_file_name = file_name.to_owned() + ".html";
        let html_file_path = Path::new("out/posts").join(html_file_name);
        fs::write(html_file_path, html_string).expect("Error writing HTML file");
    }
}

fn generate_list() {
    let mut entries: Vec<(String, String, String)> = Vec::new();
    for entry in Path::new("posts")
        .read_dir()
        .expect("Error reading posts directory")
    {
        let file_path = entry.expect("Error reading entry").path();
        let file_name = file_path
            .file_stem()
            .expect("Error getting file stem")
            .to_str()
            .expect("Error converting file stem to string");
        let title = extract_title(&file_path);
        let date = extract_date(&file_path);
        let date_copy = date.clone();
        entries.push((date_copy, title.to_string(), file_name.to_string()));
    }
    entries.sort_by(|a, b| b.0.cmp(&a.0));
    let liquid = liquid::ParserBuilder::with_stdlib().build().unwrap();
    let template = liquid
        .parse(
            &fs::read_to_string(".terb/template/list.liquid").expect("Error reading template file"),
        )
        .expect("Error parsing template file");
    let config: toml::Value =
        toml::from_str(&fs::read_to_string(".terb/config.toml").unwrap()).unwrap();
    let blogtitle = config["blogtitle"].as_str().unwrap();
    let description = config["description"].as_str().unwrap();
    let author = config["author"].as_str().unwrap();
    let mut list = vec![];
    for entry in entries {
        let entry_data = liquid::object!({
        "date": entry.0,
        "title": entry.1,
        "path": entry.2
        });
        list.push(entry_data);
    }
    let data = liquid::object!({
    "blogtitle": blogtitle,
    "description": description,
    "author": author,
    "list": list
    });
    let html_string = template.render(&data).expect("Error rendering template");
    let html_file_path = Path::new(config["list_path"].as_str().unwrap());
    fs::write(html_file_path, html_string).expect("Error writing HTML file");
}
