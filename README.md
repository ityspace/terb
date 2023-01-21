# terb

A terminal-based blog generator.

Version ***0.0.2***

## Features

- Creates a blog using a simple command line interface
- Uses Liquid templates for customization
- Generates a list of posts and individual post pages
- Built with Rust for performance and reliability

## Dev

```
curl https://sh.rustup.rs -sSf | sh

cargo run

cargo build
```

## How to use

```
Usage: terb COMMAND
-I Init Blog
-G Generate Blog
-S Run Web Server
```                  

Crate .terb/template/list.liquid for generate post list. An example here:

```html
<!DOCTYPE html>
<html>
<head>
<title>{{ blogtitle }}</title>
</head>
<body>
<p> I'm {{ author }}.</p>
<p> {{ description }}</p>
<ul class="list">
  {% for entry in list %}
    <li><a href="/posts/{{ entry.path }}.html">{{ entry.date }} - {{ entry.title }} </a></li>
  {% endfor %}
</ul>
</body>
</html>

```

Crate .terb/template/post.liquid for article page. An example here:

```html
<!DOCTYPE html>
<html>
<head>
<title>{{ title }}</title>
</head>
<body>
<article>{{ content }}</article>
</body>
</html>

```

The list_path attribute in config determines the location of the output list with articles, you can type out/index.html

## Contribution

Please open an issue or a pull request if you wish to contribute to the project.
