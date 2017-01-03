FROM ubuntu:16.04
MAINTAINER Tyler Levine <tyler@tylerlevine.com>

# install dependencies
RUN apt-get update && apt-get install -y \
binutils         \
curl             \
gcc              \
grub             \
grub-pc-bin      \
nasm             \
qemu             \
rake             \
xorriso          \
&& apt-get clean \
&& rm -rf /var/lib/apt/lists/* /var/cache/apt/*

ADD . /rose
WORKDIR /rose

# install rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
ENV PATH=/root/.cargo/bin:$PATH

# clean the build environment in case we're building from a dirty one
RUN rake clean

CMD ["rake"]

