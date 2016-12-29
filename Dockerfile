FROM ubuntu:16.04
MAINTAINER Tyler Levine <tyler@tylerlevine.com>

# install dependencies
RUN apt-get update && apt-get install -y \
binutils         \
curl             \
grub             \
grub-pc-bin      \
nasm             \
qemu             \
rake             \
xorriso          \
&& apt-get clean \
&& rm -rf /var/lib/apt/lists/* /var/cache/apt/*

# install rust
RUN /bin/bash -c "curl https://sh.rustup.rs -sSf | sh -s -- -y && source $HOME/.cargo/env"

ADD . /rose

WORKDIR /rose

CMD ["rake"]

