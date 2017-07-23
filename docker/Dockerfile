FROM ubuntu:17.04

ENV REDOX_TOOLCHAIN_APT http://static.redox-os.org/toolchain/apt/
ENV SRC_PATH /src
ENV CARGO_HOME /cargo
ENV RUSTUP_HOME /rustup
ENV PATH $CARGO_HOME/bin:$PATH

RUN apt-get update \
      && apt-get install -y git gosu gcc fuse nasm qemu-utils pkg-config \
         libfuse-dev make curl file sudo apt-transport-https autoconf flex \
         bison texinfo \
      && mkdir -p $CARGO_HOME \
      && mkdir -p $RUSTUP_HOME \
      && curl https://sh.rustup.rs > sh.rustup.rs \
      && sh sh.rustup.rs -y \
      && rustup update \
      && rustup component add rust-src \
      && rustup default nightly \
      && echo "deb $REDOX_TOOLCHAIN_APT /" >> /etc/apt/sources.list.d/redox.list \
      && apt-get update \
      && apt-get install -y --force-yes x86-64-elf-redox-newlib x86-64-elf-redox-binutils x86-64-elf-redox-gcc \
      && curl -O https://ftp.gnu.org/gnu/automake/automake-1.15.1.tar.gz \
      && tar -xvpf automake-1.15.1.tar.gz; cd automake-1.15.1; ./configure; make; make install; cd .. \
      && cargo install xargo \
      && cargo install cargo-config \
      && mkdir -p "$SRC_PATH"

WORKDIR $SRC_PATH

COPY entrypoint.sh /usr/local/bin/entrypoint.sh

RUN chmod +x /usr/local/bin/entrypoint.sh

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
