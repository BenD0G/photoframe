FROM ghcr.io/cross-rs/armv7-unknown-linux-gnueabihf

# From https://capnfabs.net/posts/cross-compiling-rust-apps-raspberry-pi/
RUN apt-get update
RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt-get install --assume-yes libssl-dev:armhf

ENV PKG_CONFIG_LIBDIR_armv7_unknown_linux_gnueabihf=/usr/lib/arm-linux-gnueabihf/pkgconfig