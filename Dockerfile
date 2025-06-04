FROM rust:1.86.0

RUN apt-get update && apt-get install -y \
    # tig is great for viewing git history
    tig \
    # less is needed e.g. for 'git diff' to make the output scrollable
    less

ARG USER
ARG UID
RUN useradd -m -s /bin/bash -u ${UID:-2222} $USER
USER ${USER}

RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/download/v0.25.0/probe-rs-tools-installer.sh | sh

# run this as sudo on the host machine to make st-link accessible
# curl -o /etc/udev/rules.d/69-probe-rs.rules https://probe.rs/files/69-probe-rs.rules

RUN cargo install cargo-tarpaulin

RUN rustup target add thumbv7em-none-eabihf

WORKDIR /home/$USER/dependencies_fetch_project/dummy/l6360
RUN cargo init
COPY ./l6360/Cargo.toml .
WORKDIR /home/$USER/dependencies_fetch_project/dummy/iol
RUN cargo init
COPY ./iol/Cargo.toml .
WORKDIR /home/$USER/dependencies_fetch_project/dummy/examples/stm32f446re
RUN cargo init
COPY ./examples/stm32f446re/Cargo.toml .
RUN cargo fetch
# - For example when used as devcontainer, the UID is set to a default value (see above).
#   I wasn't able to pass the UID of the local user to the container in this case.
#   So when using the devcontainer, the local user is then used and can't access the fetched dependencies.
#   To solve this, the fetched dependencies are made readable and writable by anyone.
RUN chmod -R a+rw /usr/local/cargo/registry
