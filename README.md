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

### Configuration

A valid JSON configuration file is required to run this fileserver.

```JSON
{
	"directory": "./demo",
	"host_and_port": "127.0.0.1:4000",
	"content_encodings": ["gzip", "deflate", "br", "zstd"],
	"filepath_404s": [
		["./demo/404.gz", "gzip"],
		["./demo/404.html", null]
	]
}
```

### Run

Bash the following command to serve files in the `cwd` at `localhost:4000`:

```sh
file_server path/to/config.json
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

### Range requests

`File_server` supports range requests and multipart range requests.


## Licence

BSD 3-Clause License
