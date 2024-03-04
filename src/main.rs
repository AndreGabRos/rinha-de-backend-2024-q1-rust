mod models;

use std::env;
use dotenvy::dotenv;
use actix_web::{get, post, web::{self, Data, Bytes}, App, HttpResponse, HttpServer, Responder, http};
use deadpool_postgres::{Runtime, GenericClient};
use crate::models::{NovaTransacao, RequestTransacao, RespostaTransacao, TransacaoRespostaExtrato};
use serde_json::json;
use tokio_postgres::NoTls;
use chrono::{Local,SecondsFormat::Micros};


#[post("/clientes/{id}/transacoes")]
async fn transacao(
    path: web::Path<i32>,
    transacao: Bytes,
    connection: web::Data<deadpool_postgres::Pool>
    ) -> impl Responder {

    let transacao: RequestTransacao = match serde_json::from_slice(&transacao) {
        Ok(tr) => tr,
        Err(_) => return HttpResponse::build(http::StatusCode::UNPROCESSABLE_ENTITY).body(""),
        
    };

    let connection = connection.get().await.expect("error connecting to postgres");

    if transacao.descricao.len() > 10 || transacao.descricao.len() == 0 {
        return HttpResponse::build(http::StatusCode::UNPROCESSABLE_ENTITY).body("tipo de transação inválido");
    }

    let cliente = connection.query(
        "SELECT saldo, limite FROM clientes WHERE id = $1", 
        &[&path.abs()])
        .await
        .unwrap();

    if cliente.is_empty() {
        return HttpResponse::build(http::StatusCode::NOT_FOUND).body("Cliente não encontrado.");
    }

    let row = &cliente[0];
    let saldo: i32 = row.get(0);
    let limite: i32 = row.get(1);

    let novo_saldo: i32;

    if transacao.tipo == "d" {
        if (saldo - transacao.valor) < (limite * -1) {
            return HttpResponse::build(actix_web::http::StatusCode::UNPROCESSABLE_ENTITY)
                .body("não há limite o suficiente.");
        }
        novo_saldo = saldo - transacao.valor;
    } else if transacao.tipo == "c" {
        novo_saldo = saldo + transacao.valor;
    } else {
        return HttpResponse::build(actix_web::http::StatusCode::UNPROCESSABLE_ENTITY)
            .body("Tipo inválido.");
    }

    let nova_transacao = NovaTransacao {
        id_cliente: path.abs(),
        valor: transacao.valor,
        tipo: &transacao.tipo,
        descricao: &transacao.descricao,
        realizada_em: Local::now().to_rfc3339_opts(Micros,true),
    };


    let _tr = connection.query(
        "INSERT INTO transacoes (id_cliente, valor, tipo, descricao, realizada_em) VALUES ($1, $2, $3, $4, $5)",
        &[&nova_transacao.id_cliente, &nova_transacao.valor, &nova_transacao.tipo, &nova_transacao.descricao, &nova_transacao.realizada_em])
        .await;
    
    connection.query(
        "UPDATE clientes SET saldo = $1 WHERE id = $2",
        &[&novo_saldo, &path.abs()]
    )
        .await
        .unwrap();

    let resposta = RespostaTransacao {
        saldo: novo_saldo,
        limite,
    };

    HttpResponse::Ok().json(json!(resposta))}

#[get("/clientes/{id}/extrato")]
async fn extrato(path: web::Path<i32>, connection: web::Data<deadpool_postgres::Pool>) -> impl Responder {
    
    if path.abs() > 5 || path.abs() < 0 {
        return HttpResponse::build(http::StatusCode::NOT_FOUND).body("rapaiz");
    }

    let connection = connection.get().await.expect("error connecting to postgres");

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
        let tr = TransacaoRespostaExtrato {
            valor: i.get(0),
            tipo: i.get(1),
            descricao: i.get(2),
            realizad_em: i.get(3),
        };

        v.push(tr);


    }
    
    let response_body = json!({
        "saldo": {
            "total": saldo,
            "data_extrato":  Local::now().to_rfc3339_opts(Micros,true),
            "limite": limite,
            
        },
        "ultimas_transacoes": &v,
    });
    
    HttpResponse::Ok().json(response_body)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let mut pg_config = deadpool_postgres::Config::new();
    pg_config.user = Some(env::var("POSTGRES_USER").expect("Failed to read POSTGRES_USER env var"));
    pg_config.host = Some(env::var("DB_HOSTNAME").expect("Failed to read DB_HOSTNAME env var"));
    pg_config.password = Some(env::var("POSTGRES_PASSWORD").expect("Failed to read POSTGRES_PASSWORD env var"));
    pg_config.dbname = Some(env::var("POSTGRES_DB").expect("Failed to read POSTGRES_DB env var"));

    pg_config.pool = deadpool_postgres::PoolConfig::new(100).into();

    let pg_pool = pg_config.create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("error creating pool");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pg_pool.clone()))
            .service(transacao)
            .service(extrato)
    })
    .workers(1)
    .bind(("0.0.0.0",8000))?
    .run()
    .await
}
