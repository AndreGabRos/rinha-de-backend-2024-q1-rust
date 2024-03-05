FROM rust:1.76.0

ARG timezone 
ENV TIMEZONE=${timezone:-"America/Sao_Paulo"} 

ENV DOCKERIZE_VERSION v0.7.0

RUN apt-get update \
    && apt-get install -y wget \
    && wget -O - https://github.com/jwilder/dockerize/releases/download/$DOCKERIZE_VERSION/dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz | tar xzf - -C /usr/local/bin \
    && apt-get autoremove -yqq --purge wget && rm -rf /var/lib/apt/lists/*

RUN apt update && cargo new rinha24
WORKDIR /rinha24

COPY Cargo.toml ./Cargo.toml
COPY src/ ./src/
#Adicionar o --release para os testes
RUN cargo build --release

EXPOSE 8000
#Mudar de debug para release para os testes
CMD dockerize -wait tcp://db:5432 -timeout 60m ./target/release/rinha24
