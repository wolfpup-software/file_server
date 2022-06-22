# File-server

A file-server written in rust.

## Install and run file-server

Execute the following to install `file-server`.

```
git clone https://github.com/herebythere/file-server
cd file-server/v0.1/file-server
cargo install .
file-server ../../file-server.json.example
```

Open a browser and visit `http://localhost:3000`.

## Configure the file-server

The first argument of `file-server` is the `filepath` of a `json` configuraiton file.

The configuration file uses to the following schema:

```json
{
  "port": "<number>",
  "directory": "<dir>",
  "filepath_403": "<filepath>",
  "filepath_404": "<filepath>",
  "filepath_500": "<filepath>",
}
```

Change the `port` property serve from a different port.

Change the `directory` property to target a different directory.

Each `directory` must include a `filepath` for the following status codes:
- `403` at `filepath_403`
- `404` at `filepath_404`
- `500` at `filepath_500`

All `filepath`s must be a descendant of `directory`.

## Run custom file-server configuration

To run a `file-server` with a custom configuration enter the following command.
Make sure to replace `./file-server.json` with the filepath of the custom
`json` configuration file.

```
file-server ./file-server.json
```

Open a browser and visit `http://localhost:<port>`. 

## Licence

BSD 3-Clause License

