# 编译
FROM --platform=$TARGETPLATFORM rust:1.71.0-bullseye as builder

WORKDIR /usr/src

RUN USER=root cargo new dm-ticket

RUN apt update \
    && apt install -y upx git
    
COPY Cargo.toml Cargo.lock /usr/src/dm-ticket/

COPY .cargo /usr/src/dm-ticket/.cargo

WORKDIR /usr/src/dm-ticket

RUN mkdir src/bin && cat src/main.rs > src/bin/client.rs && cat src/main.rs > src/bin/server.rs

RUN cargo build --release --verbose

COPY src /usr/src/dm-ticket/src/

RUN RUST_BACKTRACE=1 cargo build --release --verbose --bin dm-server && upx /usr/src/dm-ticket/target/release/dm-server


RUN RUST_BACKTRACE=1 cargo build --release --verbose --bin dm-client && upx /usr/src/dm-ticket/target/release/dm-client


FROM --platform=$TARGETPLATFORM debian:bullseye as runtime

ENV TZ=Asia/Shanghai

COPY scripts/start.sh /usr/bin/start

RUN apt update  \
    && apt install -y tzdata supervisor redis chromium-driver procps \
    && echo "${TZ}" > /etc/timezone \
    && ln -sf /usr/share/zoneinfo/${TZ} /etc/localtime \
    && apt clean \ 
    && rm -rf /var/cache/apt/* \
    && chmod 755 /usr/bin/start

WORKDIR /src/

COPY --from=builder /usr/src/dm-ticket/target/release/dm-server /usr/bin/dm-server

COPY --from=builder /usr/src/dm-ticket/target/release/dm-client /usr/bin/dm-client

COPY supervisord.conf /etc/supervisord.conf


CMD ["/usr/bin/start"]