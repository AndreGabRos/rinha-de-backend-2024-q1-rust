use std::env;
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use actix_web::web::{self, Data, Bytes};
use deadpool_postgres::{Runtime, GenericClient};
use serde_json::json;
use tokio_postgres::NoTls;
use chrono::{Local, SecondsFormat::Micros, NaiveDateTime};

use crate::models::{RespostaTransacao, RequestTransacao, TransacaoRespostaExtrato};

mod models;

#[post("/clientes/{id}/transacoes")]
async fn transacao(
    path: web::Path<i32>,
    transacao: Bytes,
    connection: web::Data<deadpool_postgres::Pool>
    ) -> impl Responder {

    let transacao: RequestTransacao = match serde_json::from_slice(&transacao) {
        Ok(tr) => tr,
        Err(_) => return HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY)
            .body(""),
    };

    let connection = connection.get().await.expect("erro ao conectar ao banco");

    if path.abs() >= 6 {
        return HttpResponse::build(StatusCode::NOT_FOUND)
            .body("cliente não encontrado.");
    }

    if transacao.descricao.len() > 10 || transacao.descricao.len() == 0 {
        return HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY)
            .body("descricao inválida");
    }

    let res = connection.query(
        "CALL fazer_transacao($1, $2, $3, $4);",
        &[&path.abs(), &transacao.valor, &transacao.tipo, &transacao.descricao]
    ).await;

    match res {
        Ok(a) => {
            let row = &a[0];
            let saldo: i32 = row.get(0);
            let limite: i32 = row.get(1);
            let resposta = RespostaTransacao {
                limite, saldo,
            };

            HttpResponse::Ok().json(json!(resposta)) 
        },
        Err(_) => HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY)
            .body("limite ultrapassado"),
    }
}

#[get("/clientes/{id}/extrato")]
async fn extrato(path: web::Path<i32>, connection: web::Data<deadpool_postgres::Pool>) -> impl Responder {
    if path.abs() >= 6 {
        return HttpResponse::build(StatusCode::NOT_FOUND)
            .body("rapaiz");
    }

    let connection = connection.get().await.expect("erro ao conectar ao banco");

    let sql1 = "SELECT saldo, limite FROM clientes WHERE id = $1";

    let cliente = connection.query(
        sql1, 
        &[&path.abs()])
        .await
        .unwrap();

    let row = &cliente[0];
    let saldo: i32 = row.get(0);
    let limite: i32 = row.get(1);
    let sql2 = "SELECT valor, tipo, descricao, realizada_em
        FROM transacoes
        WHERE id_cliente = $1
        ORDER BY id DESC
        LIMIT 2";

    let transacoes = connection.query(
        sql2,
        &[&path.abs()])
        .await
        .unwrap();


    let mut v = Vec::new();
    for i in &transacoes {
        let a: NaiveDateTime = i.get(3);
        let b = a.to_string();
        let tr = TransacaoRespostaExtrato {
            valor: i.get(0),
            tipo: i.get(1),
            descricao: i.get(2),
            realizad_em: b,
        };

        v.push(tr);
    }
    
    let response_body = json!({
        "saldo": {
            "total": saldo,
            "data_extrato": Local::now().to_rfc3339_opts(Micros,true),
            "limite": limite,
            
        },
        "ultimas_transacoes": &v,
    });
    
    HttpResponse::Ok().json(response_body)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut pg_config = deadpool_postgres::Config::new();
    pg_config.user = Some(env::var("POSTGRES_USER").expect("Failed to reade POSTGRES_USER env var"));
    pg_config.host = Some(env::var("DB_HOSTNAME").expect("Failed to reade DB_HOSTNAME env var"));
    pg_config.password = Some(env::var("POSTGRES_PASSWORD").expect("Failed to reade POSTGRES_PASSWORD env var"));
    pg_config.dbname = Some(env::var("POSTGRES_DB").expect("Failed to reade POSTGRES_DB env var"));

    pg_config.pool = deadpool_postgres::PoolConfig::new(70).into();

    let pg_pool = pg_config.create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("erro criando o pool");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pg_pool.clone()))
            .service(transacao)
            .service(extrato)
    })
    .bind(("0.0.0.0",8000))?
    .run()
    .await
}
