curr_dir=`dirname $0`
config_path=$curr_dir/config
file_server_path=$curr_dir/file-server
podmanfile_path=$curr_dir/container/file-server.podmanfile
podman_compose_path=$curr_dir/container/podman-compose.yml.template
container_path=$curr_dir/container/Cargo.toml

echo $config_path
echo $file_server_path
echo $container_path
echo $podman_compose_path


# if destination does not exist, don't make anything
if ! [ -d $1 ]; then
    echo "error: \$1 destination does not exist"
    exit 1
fi

# only copy config if necessary
if ! [ -d $1/config ]; then
    cp -r $config_path $1
fi

# only copy file_server if necessary
if ! [ -d $1/file-server ]; then
    cp -r $file_server_path $1
fi

cp $podmanfile_path $1


# ($1: destination) ($2: config_filepath) ($3: podman-compose template)
cargo run --manifest-path $container_path $1 $2 $podman_compose_path