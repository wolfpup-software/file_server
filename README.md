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
