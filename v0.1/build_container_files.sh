# build_container_files
# brian taylor vann
#
# args ($1: destination) ($2: config_filepath)


curr_dir=`dirname $0`

config_path=$curr_dir/config
file_server_path=$curr_dir/file_server
podmanfile_path=$curr_dir/templates/file_server.podmanfile
podman_compose_path=$curr_dir/templates/podman-compose.yml.template
podmanfile_path=$curr_dir/templates/podmanfile.template
container_path=$curr_dir/container/Cargo.toml

echo curr_dir

# if destination does not exist, don't make anything
if ! [ -d $1 ]; then
    echo "error: \$1 destination does not exist"
    exit 1
fi

# ($1: destination) ($2: config_filepath)
cargo run --manifest-path $container_path $1 $2 $podmanfile_path $podman_compose_path

