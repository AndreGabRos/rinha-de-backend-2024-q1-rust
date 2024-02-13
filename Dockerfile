FROM rust:1.76.0

ARG timezone 
ENV TIMEZONE=${timezone:-"America/Sao_Paulo"} 

RUN apt update && cargo new rinha24
WORKDIR /rinha24

COPY Cargo.toml .
COPY src/ ./src/
#RUN cargo build --release  

EXPOSE 8000
#Trocar o comando do conatiner de docker run para a linha de baixo durante os testes de performance
#CMD ./target/release/rinha24
CMD cargo run