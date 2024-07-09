# Configuration file to setup the Docker container of Redox

FROM rustlang/rust:nightly

ENV IMAGE_NAME=redox-os-docker

RUN set -ex;                                                                   \
    apt-get update;                                                            \
    apt-get install -q -y --no-install-recommends                              \
        apt-transport-https                                                    \
        bison                                                                  \
        flex                                                                   \
        fuse                                                                   \
        gosu                                                                   \
        libfuse-dev                                                            \
        nasm                                                                   \
        qemu-utils                                                             \
        sudo                                                                   \
        texinfo                                                                \
        autopoint                                                              \
        git                                                                    \
        cmake                                                                  \
        gperf                                                                  \
        libhtml-parser-perl                                                    \
        ;                                                                      \
    apt-key adv --keyserver keyserver.ubuntu.com --recv-keys AA12E97F0881517F; \
    echo "deb https://static.redox-os.org/toolchain/apt/ /" >>                 \
        /etc/apt/sources.list.d/redox.list;                                    \
    apt-get update -o Dir::Etc::sourcelist="redox.list";                       \
    apt-get install -q -y --no-install-recommends                              \
        x86-64-unknown-redox-newlib                                            \
        x86-64-unknown-redox-binutils                                          \
        x86-64-unknown-redox-gcc                                               \
        ;                                                                      \
    cargo install cargo-config;                                                \
    cargo install just@1.16.0;                                                 \
    apt-get autoremove -q -y;                                                  \
    apt-get clean -q -y;                                                       \
    rm -rf /var/lib/apt/lists/*

COPY entrypoint.sh /usr/local/bin/
COPY .bash_aliases /etc/skel/

ENTRYPOINT ["bash", "/usr/local/bin/entrypoint.sh"]
CMD ["/bin/bash"]
