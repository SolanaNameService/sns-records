#!/usr/bin/env bash
# Check if the Docker image exists
set -e
docker build -t sns-records .

mkdir -p target/deploy
solana program dump namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX target/deploy/spl_name_service.so

if [[ ${1} == "build-only" ]]; then
    echo "Only building..."
    docker run -it \
        --net=host \
        --mount "type=bind,source=$(pwd),target=/workdir" \
        --mount type=bind,source=$(pwd)/../../sns-warp/common,target=/sns-warp/common,readonly \
        sns-records:latest \
        /bin/bash -c "cargo build-sbf"
elif [[ ${1} == "test" ]]; then
    echo "Running tests..."
    docker run -it \
        --net=host \
        --mount "type=bind,source=$(pwd),target=/workdir" \
        --mount type=bind,source=$(pwd)/../../sns-warp/common,target=/sns-warp/common,readonly \
        sns-records:latest \
        /bin/bash -c "cargo test-sbf"
else
    echo "Running tests + building..."
    docker run -it \
        --net=host \
        --mount "type=bind,source=$(pwd),target=/workdir" \
        --mount type=bind,source=$(pwd)/../../sns-warp/common,target=/sns-warp/common,readonly \
        sns-records:latest \
        /bin/bash -c "cargo test-sbf && cargo build-sbf"
fi