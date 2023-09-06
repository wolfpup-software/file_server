# file_server

`http` file server written in rust using [tokio](https://tokio.rs/) and
[hyper](https://hyper.rs/).

## Create a config

A JSON configuration file is required to run `hbt_file_server` or create a
`hbt_file_server` container.

Configuration files are expected to use use the following schema:

```
{
  "host": <string>,
  "port": <number>,
  "directory": <string>,
  "filepath_404": <string>,
  "filepath_500": <string>
}
```

Change the `host` property to serve from a specific host.

Change the `port` property to serve from a different port.

Change the `directory` property to target a alternative directory.

Each `directory` must include a `filepath` for the following status codes:

- `404` at `filepath_404`
- `500` at `filepath_500`

All `filepaths` must be a descendant of `directory`.

An example of a valid configuration can be found at
`file_server/v0.1/resources/file_server.json.example`

```json
{
  "host": "127.0.0.1",
  "port": 3000,
  "directory": "./docs",
  "filepath_404": "./docs/404.html",
  "filepath_500": "./docs/500.html"
}
```

## Install file_server

Execute the following to install `htb_file_server`.

```
git clone https://github.com/herebythere/file_server
cargo install --path file_server/v0.1/file_server
```

## Run file_server

The `hbt_file_server` application accepts one argument:

- A valid `hbt_file_server` JSON configuration file

The following psuedo-script shows the argument schema:

```
hbt_file_server <path_to_configuration_file>
```

Execute the following to run a `hbt_file_server` demo hosting the repositories
`/docs` directory.

```
hbt_file_server file_server/v0.1/resources/file_server.json.example
```

Open a browser and visit `http://localhost:3000`.

## file_server containers

A utility script is provided to build containers with `podman`.

The containers are built from the same configuration files as the
`hbt_file_server` application.

#### Install required software

Install `podman` and `podman-compose`:

```
dnf install podman podman-compose
```

#### Create container scripts

The container script requires two arguments:

- A destination directory for the generated files
- A valid `hbt_file_server` JSON configuration file

The following psuedo cli command shows the argument schema:

```
bash build_container_files.sh <destination directory> <json config filepath>
```

Run the following shell commands to create a container based on
`file_server.json.example`:

```
mkdir file_server/ctnr/
bash file_server/v0.1/build_container_files.sh \
  file_server/ctnr \
  file_server/v0.1/resources/file_server.json.example
```

#### Deploy container

The demo `podman` container files will be located in this repository at:
`file_server/cntr`.

To build the container:

```
podman-compose -f file_server/ctnr/file_server.podman-compose.yml build
```

To up the container:

```
podman-compose -f file_server/ctnr/file_server.podman-compose.yml up -d
```

To down the container:

```
podman-compose -f file_server/ctnr/file_server.podman-compose.yml down
```

Replace `file_server/ctnr/file_server.podman-compose.yml` with a different
filepath to target an alternative container.

## Licence

BSD 3-Clause License
