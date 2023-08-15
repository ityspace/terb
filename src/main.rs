extern crate liquid;
extern crate pulldown_cmark;

use pulldown_cmark::HeadingLevel;
use pulldown_cmark::{html, Event, Options, Parser, Tag};
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
        println!("Version 0.1.4");
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
            let keys = ["blogtitle", "description", "author", "list_path"];
            for key in &keys {
                let mut config_string = String::new();
                println!("Enter {}:", key);
                io::stdin().read_line(&mut config_string).unwrap();
                let value = config_string.trim();
                if key == &"language" {
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
            generate_template();
            println!("Finish!");
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
fn generate_template() {
    let listhtml = r#"
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>{{ blogtitle }}</title>
  <meta name="description" content="{{ description }}">
  <meta name="keywords" content="blog, thoughts, experiences">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <style>
  :root {
    --default-text-color: #333;
    --default-background-color: #fff;
    --title-color: #000;
    --border-color:#ddd;
    --link-color:#000;
    --visit-color:#444;
    --code-background-color: #eff1f3;
    --code-color: #000;
    --span-color:#708090;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      --default-text-color: #a0aaaf;
      --default-background-color: #101010;
      --title-color: #d0dadf;  
      --border-color:#808a8f50;
      --link-color:#000;
      --visit-color:#aoaaaf;
      --code-background-color: transparent;
      --code-color: #a0aaaf;
      --span-color:#a0aaaf;
    }
  }
  
  html {
    height: 100%;
  }

  body {
    line-height: calc(16px * 1.618);
    font-size: 16px;
    max-width: 80ch;
    margin: 1rem auto;
    color: var(--default-text-color);
    padding: 0 2rem 2rem 2rem;
    background: var(--default-background-color);
    font-family: ui-sans-serif,system-ui;
    word-break: break-all;
  }

  img,
  video {
    max-width: 100%;
    margin: 0.5em 0;
    border-radius: 8px
  }
  h1,
  h2,
  h3,
  h4,
  h5,
  h6 {
    line-height: 1.25;
    font-weight: normal;
    color: var(--title-color)
  }
article h1,
  article h2 {
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 0.1em
  }

  nav {
    padding: 0.67em 0 calc(0.67em * 2) 0
  }
  nav h1 {
    font-size: xx-large;
  }
  a img {
    transition: all 0.5s ease
  }

  a img:hover {
    border-radius: 0px
  }

  a {
    color: var(--link-color)
  }

  a:visited {
    color: var(--visit-color)
  }

  a:hover {
    color: var(--link-color);
    text-decoration: none
  }
        
  header,
  footer {
    font-size: 14px
  }

  hr {
    box-sizing: content-box;
    height: 0;
    border: 0;
    border-top: 1px solid var(--border-color)
  }

  ul,
  ol {
    margin-block-start: 0.25em;
    margin-block-end: 1em;
    padding-inline-start: 2.5em
  }

  ul li::marker {
    content: '- '
  }

  ul {
    padding-inline-start: 1.5em
  }

  code {
    background: var(--code-background-color);
    color: var(--code-color);
    padding: 2px 1ch;
    border-radius: 6px;
    font-family: Menlo, Consolas, Monaco, Liberation Mono, Lucida Console, monospace;
    font-size: 14px
  }

  pre code {
    background: transparent;
    display: block;
    line-height: 1.25;
    padding: 1.5em;
    overflow-x: auto;
    overflow-y: hidden;
    border: 1px solid var(--border-color)
  }

  blockquote {
    margin-inline-start: 20px;
    padding-left: 20px;
    border-left: 2px solid var(--border-color);
    font-style: italic
  }

  table {
    border-collapse: collapse;
    max-width: 100%
  }

  table tr td,
  table tr th {
    padding: 0.3em 1ch
  }

  table tr td {
    vertical-align: top
  }

  table tr th {
    border-bottom: 1px solid var(--border-color);
    color: var();
    font-weight: normal;
    text-align: inherit
  }
  footer {
    display: flex;
    justify-content: space-between;
    flex-direction: row;
    margin: 3.6rem 0;
  }
  .article-link {
    margin:3.6rem 0;
  }
  .article-link a {
    text-decoration: none;
    color: var(--title-color)
  }
  .article-link span{
    color: var(--span-color);
  }
  .article-link h2 {
    margin-bottom: 10px;
  }
  </style>
</head>
<body>
  <header>
    <nav>
      <h1>{{ blogtitle }}</h1>
    </nav>
  </header>
  <main>
<h1>Articles</h1>
{% for entry in list %}
<div class="article-link"><h2><a href="/posts/{{ entry.path }}.html">{{ entry.title }}</a></h2><span>{{ entry.date }}</span></div>
{% endfor %}
</main>
    <footer>
      <p>CC BY-NC-SA 4.0</p>
      <div><svg viewBox="0 0 605 605" width="50px" height="50px" fill="currentColor">
        <path d="M235 36c0 2 4.9 10.7 31.6 56.6 14.7 25.3 29.4 50.9 32.6 57.1l5.9 11.2-3.8 2.4c-2.1 1.3-20.9 12.3-41.8 24.4-20.9 12.2-45.8 26.6-55.4 32.2-9.6 5.6-17.5 10-17.7 9.9-.1-.2-9.9-17-21.7-37.3-31.3-54-47.3-80.2-51.5-83.9-1.3-1.1-2.2-1.4-2.2-.7 0 2.1 4.4 10.1 29.3 52.8 31.6 54.3 37 64 36.4 65.8-1.5 3.8-20.5 5-90.7 6.1-25.6.4-47.4 1-48.5 1.5-1.1.4 10.9.8 26.6.8 15.7.1 48.9.4 73.7.8l45.2.6v139.4l-27.7.6c-15.3.4-42.4.8-60.3.8-57.6.2-65.7 1.3-16.5 2.2 75.6 1.4 96.8 2.8 98.2 6.3.6 1.7-6.3 14.4-37.3 67.6-13.4 23-25.3 43.7-26.4 45.9-2.5 4.8-2.2 7 .5 4.3 4.3-4.4 26.6-40.8 55.5-90.9 9.5-16.5 17.3-30.1 17.4-30.2.1-.1 17.7 10 39.1 22.5 21.5 12.5 47.4 27.5 57.5 33.4 10.2 5.9 19.3 11.3 20.4 12.1 1.7 1.3 1.5 2-4.9 13.8-3.8 6.8-15.4 27-25.8 44.9-34.1 58.3-37.7 64.6-37.7 67.5 0 .8.4 1.5.9 1.5 1 0 11.4-16.4 28.1-44.5 25.2-42.4 42.3-70.6 43.8-72.5 1.6-2 2-1.5 11.4 13.7 5.3 8.7 17.6 29.3 27.4 45.8 20.8 35.3 32.9 54.9 34.9 56.5 1.2 1 1.5.8 1.5-1.1 0-2.6-4.1-10-29.6-53.5-20.5-35-33.9-58.4-37.7-66.1l-2.6-5.1 12.7-7.5c7-4.1 32.7-19 57.2-33.2 24.5-14.2 45.5-26.4 46.6-27.2 2-1.3 2.7-.3 19 27.9 28.7 49.9 53 89.6 56.7 92.7 2.5 2 3.1.7 1.2-2.9-.9-1.7-12.5-21.9-25.9-44.8-26.7-46-39.6-68.7-39.6-70 0-1.7 5.4-3.3 14.8-4.2 8.4-.8 72.4-2.6 113.2-3.2l13.5-.1-13.5-.8c-7.4-.3-27.7-.7-45.1-.8-17.4 0-44.5-.4-60.2-.8l-28.7-.6V236.3l45.8-.6c25.1-.4 58.3-.7 73.7-.8 15.4 0 27.1-.4 26-.8-1.1-.5-22.9-1.1-48.5-1.5-66.9-1-88.5-2.3-90.5-5.6-1-1.6 3.6-9.9 41.6-75.1 22.5-38.7 25.3-43.8 24.5-44.6-2.2-2.3-18.3 23.2-56.1 88.7-10.3 17.9-18.9 32.9-19.2 33.3-.3.4-2.3-.3-4.6-1.6-7.7-4.3-86-49.9-100.5-58.4l-14.2-8.4 1.6-3.2c4.1-7.9 18-32.3 31.7-55.7 33.9-57.6 38.9-67 35.8-67-1.3 0-12.2 17.4-34.8 55.5-21.1 35.5-34.7 58-36.9 61-1.3 1.7-1.6 1.8-2.5.5-.5-.8-10.9-18.2-23.1-38.5C244.3 45.6 237.7 35 236 35c-.5 0-1 .5-1 1zm88.3 136.8c18.3 10.9 63.3 36.8 86.7 49.9l19.5 11v144.7L380 406.7c-27.2 15.5-54.3 30.8-60.3 33.8l-10.7 5.6-13.3-7c-7.2-3.9-34.3-19.2-60.2-34l-47-26.8-.3-72-.2-72 59.7-34.6c32.9-19 60.4-34.6 61.1-34.6.7-.1 7.3 3.4 14.5 7.7z"></path>
      </svg></div>
    </footer>
  </body>
</html>
  "#;
    let posthtml = r#"
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>{{ title }}</title>
  <meta name="description" content="{{ title }}">
  <meta name="keywords" content="{{ title }}">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
   <style>
   :root {
    --default-text-color: #333;
    --default-background-color: #fff;
    --title-color: #000;
    --border-color:#ddd;
    --link-color:#000;
    --visit-color:#444;
    --code-background-color: #eff1f3;
    --code-color: #000;
    --span-color:#708090;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      --default-text-color: #a0aaaf;
      --default-background-color: #101010;
      --title-color: #d0dadf;  
      --border-color:#808a8f50;
      --link-color:#000;
      --visit-color:#a0aaaf;
      --code-background-color: transparent;
      --code-color: #a0aaaf;
      --span-color:#a0aaaf;
    }
  }
  
  html {
    height: 100%;
  }

  body {
    line-height: calc(16px * 1.618);
    font-size: 16px;
    max-width: 80ch;
    margin: 1rem auto;
    color: var(--default-text-color);
    padding: 0 2rem 2rem 2rem;
    background: var(--default-background-color);
    font-family: ui-sans-serif,system-ui;
    word-break: break-all;
  }

  img,
  video {
    max-width: 100%;
    margin: 0.5em 0;
    border-radius: 8px
  }
  h1,
  h2,
  h3,
  h4,
  h5,
  h6 {
    line-height: 1.25;
    font-weight: normal;
    color: var(--title-color)
  }
article h1,
  article h2 {
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 0.1em
  }

  nav {
    padding: 0.67em 0 calc(0.67em * 2) 0
  }
  nav h1 {
    font-size: xx-large;
  }
  a img {
    transition: all 0.5s ease
  }

  a img:hover {
    border-radius: 0px
  }

  a {
    color: var(--link-color)
  }

  a:visited {
    color: var(--visit-color)
  }

  a:hover {
    color: var(--link-color);
    text-decoration: none
  }
        
  header,
  footer {
    font-size: 14px
  }

  hr {
    box-sizing: content-box;
    height: 0;
    border: 0;
    border-top: 1px solid var(--border-color)
  }

  ul,
  ol {
    margin-block-start: 0.25em;
    margin-block-end: 1em;
    padding-inline-start: 2.5em
  }

  ul li::marker {
    content: '- '
  }

  ul {
    padding-inline-start: 1.5em
  }

  code {
    background: var(--code-background-color);
    color: var(--code-color);
    padding: 2px 1ch;
    border-radius: 6px;
    font-family: Menlo, Consolas, Monaco, Liberation Mono, Lucida Console, monospace;
    font-size: 14px
  }

  pre code {
    background: transparent;
    display: block;
    line-height: 1.25;
    padding: 1.5em;
    overflow-x: auto;
    overflow-y: hidden;
    border: 1px solid var(--border-color)
  }

  blockquote {
    margin-inline-start: 20px;
    padding-left: 20px;
    border-left: 2px solid var(--border-color);
    font-style: italic
  }

  table {
    border-collapse: collapse;
    max-width: 100%
  }

  table tr td,
  table tr th {
    padding: 0.3em 1ch
  }

  table tr td {
    vertical-align: top
  }

  table tr th {
    border-bottom: 1px solid var(--border-color);
    color: var();
    font-weight: normal;
    text-align: inherit
  }
  footer {
    display: flex;
    justify-content: space-between;
    flex-direction: row;
    margin: 3.6 0;
  }
  </style>
</head>
<body>
  <main>
   <article>
   {{ content }}
   </article>
   <a href="/">Return home</a>
  </main>
  <footer>
  <p>CC BY-NC-SA 4.0</p>
  <div><svg viewBox="0 0 605 605" width="50px" height="50px" fill="currentColor">
    <path d="M235 36c0 2 4.9 10.7 31.6 56.6 14.7 25.3 29.4 50.9 32.6 57.1l5.9 11.2-3.8 2.4c-2.1 1.3-20.9 12.3-41.8 24.4-20.9 12.2-45.8 26.6-55.4 32.2-9.6 5.6-17.5 10-17.7 9.9-.1-.2-9.9-17-21.7-37.3-31.3-54-47.3-80.2-51.5-83.9-1.3-1.1-2.2-1.4-2.2-.7 0 2.1 4.4 10.1 29.3 52.8 31.6 54.3 37 64 36.4 65.8-1.5 3.8-20.5 5-90.7 6.1-25.6.4-47.4 1-48.5 1.5-1.1.4 10.9.8 26.6.8 15.7.1 48.9.4 73.7.8l45.2.6v139.4l-27.7.6c-15.3.4-42.4.8-60.3.8-57.6.2-65.7 1.3-16.5 2.2 75.6 1.4 96.8 2.8 98.2 6.3.6 1.7-6.3 14.4-37.3 67.6-13.4 23-25.3 43.7-26.4 45.9-2.5 4.8-2.2 7 .5 4.3 4.3-4.4 26.6-40.8 55.5-90.9 9.5-16.5 17.3-30.1 17.4-30.2.1-.1 17.7 10 39.1 22.5 21.5 12.5 47.4 27.5 57.5 33.4 10.2 5.9 19.3 11.3 20.4 12.1 1.7 1.3 1.5 2-4.9 13.8-3.8 6.8-15.4 27-25.8 44.9-34.1 58.3-37.7 64.6-37.7 67.5 0 .8.4 1.5.9 1.5 1 0 11.4-16.4 28.1-44.5 25.2-42.4 42.3-70.6 43.8-72.5 1.6-2 2-1.5 11.4 13.7 5.3 8.7 17.6 29.3 27.4 45.8 20.8 35.3 32.9 54.9 34.9 56.5 1.2 1 1.5.8 1.5-1.1 0-2.6-4.1-10-29.6-53.5-20.5-35-33.9-58.4-37.7-66.1l-2.6-5.1 12.7-7.5c7-4.1 32.7-19 57.2-33.2 24.5-14.2 45.5-26.4 46.6-27.2 2-1.3 2.7-.3 19 27.9 28.7 49.9 53 89.6 56.7 92.7 2.5 2 3.1.7 1.2-2.9-.9-1.7-12.5-21.9-25.9-44.8-26.7-46-39.6-68.7-39.6-70 0-1.7 5.4-3.3 14.8-4.2 8.4-.8 72.4-2.6 113.2-3.2l13.5-.1-13.5-.8c-7.4-.3-27.7-.7-45.1-.8-17.4 0-44.5-.4-60.2-.8l-28.7-.6V236.3l45.8-.6c25.1-.4 58.3-.7 73.7-.8 15.4 0 27.1-.4 26-.8-1.1-.5-22.9-1.1-48.5-1.5-66.9-1-88.5-2.3-90.5-5.6-1-1.6 3.6-9.9 41.6-75.1 22.5-38.7 25.3-43.8 24.5-44.6-2.2-2.3-18.3 23.2-56.1 88.7-10.3 17.9-18.9 32.9-19.2 33.3-.3.4-2.3-.3-4.6-1.6-7.7-4.3-86-49.9-100.5-58.4l-14.2-8.4 1.6-3.2c4.1-7.9 18-32.3 31.7-55.7 33.9-57.6 38.9-67 35.8-67-1.3 0-12.2 17.4-34.8 55.5-21.1 35.5-34.7 58-36.9 61-1.3 1.7-1.6 1.8-2.5.5-.5-.8-10.9-18.2-23.1-38.5C244.3 45.6 237.7 35 236 35c-.5 0-1 .5-1 1zm88.3 136.8c18.3 10.9 63.3 36.8 86.7 49.9l19.5 11v144.7L380 406.7c-27.2 15.5-54.3 30.8-60.3 33.8l-10.7 5.6-13.3-7c-7.2-3.9-34.3-19.2-60.2-34l-47-26.8-.3-72-.2-72 59.7-34.6c32.9-19 60.4-34.6 61.1-34.6.7-.1 7.3 3.4 14.5 7.7z"></path>
  </svg></div>
  </footer>
</body>
</html>
      "#;

    let mut listfile = match File::create(".terb/template/list.liquid") {
        Ok(file) => file,
        Err(error) => {
            println!("Error creating file: {}", error);
            return;
        }
    };
    let mut postfile = match File::create(".terb/template/post.liquid") {
        Ok(file) => file,
        Err(error) => {
            println!("Error creating file: {}", error);
            return;
        }
    };

    match listfile.write_all(listhtml.as_bytes()) {
        Ok(_) => println!("List Template Created"),
        Err(error) => println!("Error writing to file: {}", error),
    }
    match postfile.write_all(posthtml.as_bytes()) {
        Ok(_) => println!("Post Template Created"),
        Err(error) => println!("Error writing to file: {}", error),
    }
}
