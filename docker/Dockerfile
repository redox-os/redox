FROM ubuntu:17.04

ENV REDOX_TOOLCHAIN_APT https://static.redox-os.org/toolchain/apt/

ENV USER user
ARG LOCAL_UID=local
ARG LOCAL_GID=local
ENV BUILD_UID=${LOCAL_UID:-9001}
ENV BUILD_GID=${LOCAL_GID:-9001}

RUN   apt-get update \
      && apt-get install -y dirmngr git gosu gcc fuse nasm qemu-utils pkg-config \
             libfuse-dev make curl wget file sudo apt-transport-https autoconf flex \
             bison texinfo \
      && apt-key adv --fetch-keys https://static.redox-os.org/toolchain/apt/keyFile \
      && echo "deb $REDOX_TOOLCHAIN_APT /" >> /etc/apt/sources.list.d/redox.list \
      && apt-get update -o Dir::Etc::sourcelist="redox.list" \
      && apt-get install -y x86-64-unknown-redox-newlib x86-64-unknown-redox-binutils x86-64-unknown-redox-gcc \
      && groupadd -g $BUILD_GID user \
      && useradd --shell /bin/bash -u $BUILD_UID -g $BUILD_GID -o -c "" -m $USER \
      && echo "$USER ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/user-no-sudo-password

COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

USER $USER
ENV HOME /home/$USER
ENV PATH $HOME/.cargo/bin:$PATH
ENV SRC_PATH $HOME/src
WORKDIR $HOME
RUN   curl https://sh.rustup.rs > sh.rustup.rs \
      && sh sh.rustup.rs -y \
      && rustup update \
      && rustup component add rust-src \
      && rustup default nightly \
      && curl -O https://ftp.gnu.org/gnu/automake/automake-1.15.1.tar.gz \
      && tar -xvpf automake-1.15.1.tar.gz; cd automake-1.15.1; ./configure; make; sudo make install; cd .. \
      && cargo install xargo \
      && cargo install cargo-config \
      && mkdir -p $SRC_PATH

WORKDIR $SRC_PATH
USER root

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
