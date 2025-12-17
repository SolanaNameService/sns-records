#!/usr/bin/env bash
set -v -e
docker build -t solana_build .

solana program dump namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX target/deploy/spl_name_service.so

docker run -it --mount type=bind,source=$(pwd),target=/workdir --env CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse \
--env SSH_AUTH_SOCK=/ssh-agent \
--mount type=bind,source=$SSH_AUTH_SOCK,target=/ssh-agent \
solana_build:latest cargo test-sbf
docker run -it --mount type=bind,source=$(pwd),target=/workdir --env CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse \
--env SSH_AUTH_SOCK=/ssh-agent \
--mount type=bind,source=$SSH_AUTH_SOCK,target=/ssh-agent \
solana_build:latest cargo build-sbf