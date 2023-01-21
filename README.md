# terb

*Blog generator in Terminal.*

***Version 0.0.1***

## Dev

```
curl https://sh.rustup.rs -sSf | sh
```

```cargo build```

```cargo run```

## How to use

```
Usage: terb COMMAND
-I Init Blog
-G Generate Blog
-S Run Web Server
```                  

You need to crate template/list.html as template file for generate post list. An example here:

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

You need to crate template/post.html as template file for article page. An example here:

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
