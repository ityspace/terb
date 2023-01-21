extern crate liquid;
extern crate pulldown_cmark;

use pulldown_cmark::HeadingLevel;
use pulldown_cmark::{html, Event, Parser, Tag};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
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
        println!("Version 0.0.1");
        return;
    }
    let command = &args[1];
    match command.as_ref() {
        "-G" => {
            generate_posts();
            generate_list();
            println!("Success!")
        }
        "-S" => {
            println!("Serving HTTP on 127.0.0.1 port 7878 (http://localhost:7878/)");
            println!("Press Ctrl+C to exit.");
            let file_path = Path::new("static/404.html");
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
            let dirs = ["posts", "static", "static/posts"];
            for dir in &dirs {
                match fs::create_dir_all(dir) {
                    Ok(_) => println!("{} directory created", dir),
                    Err(e) => println!("Error creating {} directory: {:?}", dir, e),
                }
            }
            println!("Finish!");
            let mut config = toml::value::Table::new();
            let keys = ["blogtitle", "description", "author", "list_path"];
            for key in &keys {
                let mut config_string = String::new();
                println!("Enter {}:", key);
                io::stdin().read_line(&mut config_string).unwrap();
                let value = config_string.trim();
                config.insert(key.to_string(), toml::Value::String(value.to_string()));
            }
            let config_string = toml::to_string(&toml::Value::Table(config)).unwrap();
            let mut file = File::create("config.toml").unwrap();
            file.write_all(config_string.as_bytes()).unwrap();
            println!("Finish!");
        }
        _ => println!("Invalid command"),
    }
}
fn check_dir(path: &str) {
    let path = Path::new(path);
    if !path.exists() {
        println!(
            "Directory '{}' not found, create now? (Y/N)",
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
            ("HTTP/1.1 200 OK\r\n\r\n", "static/index.html".to_string())
        } else {
            let requested_path = requested_path[1..].to_string();
            let file_path = Path::new("static").join(requested_path);
            if file_path.exists() {
                (
                    "HTTP/1.1 200 OK\r\n\r\n",
                    file_path.to_str().unwrap().to_string(),
                )
            } else {
                (
                    "HTTP/1.1 404 NOT FOUND\r\n\r\n",
                    "static/404.html".to_string(),
                )
            }
        }
    } else {
        (
            "HTTP/1.1 404 NOT FOUND\r\n\r\n",
            "static/404.html".to_string(),
        )
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
    let parser = Parser::new(&md_string);
    let mut html_string = String::new();
    html::push_html(&mut html_string, parser);
    html_string
}
fn generate_posts() {
    check_dir("./static");
    check_dir("./static/posts");
    check_dir("./posts");
    let posts_dir = Path::new("posts");
    let liquid = liquid::ParserBuilder::with_stdlib().build().unwrap();
    let template = liquid
        .parse(&fs::read_to_string("template/post.html").expect("Error reading template file"))
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
        let html_file_path = Path::new("static/posts").join(html_file_name);
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
        .parse(&fs::read_to_string("template/list.html").expect("Error reading template file"))
        .expect("Error parsing template file");
    let config: toml::Value = toml::from_str(&fs::read_to_string("config.toml").unwrap()).unwrap();
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
