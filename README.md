# file-server

An `http` file server written in rust using [tokio](https://tokio.rs/) and [hyper](https://hyper.rs/).

## Create a config

A JSON configuration file is required to run `file-server` or create a `file-server` container.

Configuration files are expected to use use the following schema:

```json
{
  "port": "<number>",
  "directory": "<string>",
  "filepath_403": "<string>",
  "filepath_404": "<string>",
  "filepath_500": "<string>",
}
```

Change the `port` property to serve from a different port.

Change the `directory` property to target a different directory.

Each `directory` must include a `filepath` for the following status codes:
- `403` at `filepath_403`
- `404` at `filepath_404`
- `500` at `filepath_500`

All `filepaths` must be a descendant of `directory`.

An example of a valid configuration can be found at the top level
of this repository: `file-server.json.example`

```json
{
    "port": 3000,
    "directory": "./demo",
    "filepath_403": "./demo/403.html",
    "filepath_404": "./demo/404.html",
    "filepath_500": "./demo/500.html"
}
```

## Install file-server

Execute the following to install `file-server`.

```
git clone https://github.com/herebythere/file-server
cd file-server/v0.1/file-server
cargo install .
```

## Run file-server

`file-server` accepts one argument:
- A valid `file-server` JSON configuration file

The following psuedo-script shows the argument schema:
```
file-server <path_to_configuration_file>
```

Execute the following to run the `file-server` demo.
```
file-server ../../file-server.json.example
```

Open a browser and visit `http://localhost:3000`.

## File-server containers

A utility script is provided to build containers with `podman`.

The containers are built from the same configuration files as the `file-server` application.

#### Install required software

Install `podman` and `podman-compose`:

```
dnf install podman podman-compose
```

#### Create container scripts

Move to the following repository directory: `file-server/v0.1/container`.

The container script requires two arguments:
- A valid `file-server` JSON configuration file
- A destination directory for the generated files

The following psuedo-script shows the argument schema
```
cargo run <path_to_config> <destination_directory>
```

Run the following script to create a container that serves the `file-server` demo:

```
cargo run ../../file-server.json.example ../../demo
```

#### Deploy container

Move to the `<destination_directory>` containing the generated  files.

The demo `podman` files will be located in this repository at:
`file-server/demo`.

There should be three files:
- A JSON configuration file to configure the `file-server` container
- A `file-server.podmanfile` to build the container image
- A `file-server.podman-compose.yml` to deploy the container

To build the container:
```
podman-compose -f ./file-server.podman-compose.yml build
```

To up the container:
```
podman-compose -f ./file-server.podman-compose.yml up -d
```

To down the container:
```
podman-compose -f ./file-server.podman-compose.yml down
```

## Licence

BSD 3-Clause License

