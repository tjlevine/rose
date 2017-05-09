FROM ubuntu:16.04
MAINTAINER Tyler Levine <tyler@tylerlevine.com>

# install dependencies
RUN apt-get update && apt-get install --no-install-recommends -y \
curl             \
gcc              \
grub             \
grub-pc-bin      \
libc6-dev        \
nasm             \
qemu             \
rake             \
xorriso

# install rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
ENV PATH=/root/.cargo/bin:$PATH

COPY . /rose
WORKDIR /rose

# clean the build environment in case we're building from a dirty one, then make the iso
RUN rake clean iso

# clean up the left over build tools
RUN apt-get purge -y \
binutils    \
curl        \
gcc         \
grub        \
grub-pc-bin \
nasm        \
xorriso     \
&& apt-get autoremove -y \
&& apt-get clean \
&& rm -rf /var/lib/apt/lists/* /var/cache/apt/*

CMD ["rake"]
