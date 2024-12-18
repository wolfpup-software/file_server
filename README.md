[TODO]: create a list of paths to serve
[
	'index.gzip',
	'index.html',
	'404.html',
]

if there was a configuration?
{
	root_dir: "../yo",
	encoded: [".gzip", ".br"],
	404_filepath: "../optional404.html",
	500_filepath: "../optional505.html",
}

What is need from the request?


get request
get path

create paths_and_encodings

if 404 is available add to paths_and_encodings

add path to paths_and_encodings

check accept-encodings from request
compare against available encodings

if available add to paths_and_encodings



service can have paths
service could have encodings as well
removes encodings from arc relationship
relatively cheap

paths: [
	(index.gz, Some("gzip")),
	(index.html, None),
	(404.html, None)
]

config has 404
config has 500



if not found and optional 404
return 404.html

if not found
return hard code 404

if server error:
return 500.html

if 500.html not found
return hard code 500


# file_server

An `http` file server written in rust using [tokio](https://tokio.rs/) and
[hyper](https://hyper.rs/).

## How to use

### Install

Bash the following commands:

```sh
git clone https://github.com/herebythere/file_server
cargo install --path file_server
```

### Run 

Bash the following command to serve files in the `cwd` at `localhost:3000`:

```sh
file_server localhost:3000
```

Open a browser and visit `http://localhost:3000`.

### Accept-Encoding

If a request has an `accept-encoding` header:

```
Accept-Encoding: gzip;
```

`file_server` will return a corresponding `gzip`-ed version of a requested file if available.

```sh
/www/index.html.gz	# accept-encoding: gzip;
/www/index.html		# defacto file served
```

Otherwise it will serve the unencoded file.

## Licence

BSD 3-Clause License
