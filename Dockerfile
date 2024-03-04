FROM rust:1.76.0

ARG timezone 
ENV TIMEZONE=${timezone:-"America/Sao_Paulo"} 

RUN apt update && cargo new rinha24
WORKDIR /rinha24

COPY Cargo.toml ./Cargo.toml
COPY src/ ./src/
#Adicionar o --release para os testes
RUN cargo build --release

EXPOSE 8000
#Mudar de debug para release para os testes
CMD ./target/release/rinha24
