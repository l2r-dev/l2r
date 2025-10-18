FROM rust:1.87 AS builder

WORKDIR /app

RUN apt update && apt install -y \
    libasound2-dev \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup install nightly

COPY . .
RUN mkdir build
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release \
    # Need to copy the files to the build directory, cause target in cache
    && cp target/release/l2r_gameserver \
    target/release/l2r_loginserver \
    /app/build/


FROM rust:1.87 AS runner
RUN cargo install sea-orm-cli@1.1.11

### Game Server
FROM runner AS gameserver
ARG L2R_HOME=/opt/l2r/
ENV L2R_HOME=${L2R_HOME}
WORKDIR ${L2R_HOME}
COPY ./game_server/data ${L2R_HOME}data
COPY --from=builder /app/build/l2r_gameserver ${L2R_HOME}l2r_gameserver
ENV BEVY_ASSET_ROOT="${L2R_HOME}"
ENV LUA_PATH="${L2R_HOME}?.lua;"
ENV RUST_BACKTRACE=1
ENV RUST_LOG="l2r_gameserver=debug,l2r_core=debug,bevy_asset=debug,wgpu_core=off,sqlx=off,bevy_mod_scripting=debug,sea_orm_migration=debug"
CMD ["./l2r_gameserver"]


### Login Server
FROM runner AS loginserver
ARG L2R_HOME=/opt/l2r/
ENV L2R_HOME=${L2R_HOME}
ENV BEVY_ASSET_ROOT="${L2R_HOME}"
ENV RUST_BACKTRACE=1
ENV RUST_LOG="l2r_loginserver=debug,l2r_core=debug,sqlx=off,sea_orm_migration=debug,bevy_ecs=debug"


WORKDIR ${L2R_HOME}
COPY ./data ${L2R_HOME}data
COPY --from=builder /app/build/l2r_loginserver ${L2R_HOME}l2r_loginserver
CMD ["./l2r_loginserver"]