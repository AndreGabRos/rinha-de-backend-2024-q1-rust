use core::panic;
use std::sync::Arc;
use std::env;
use chrono::{DateTime, Utc};
use dotenvy::dotenv;
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use actix_web::web::{self, Data, Bytes};
use serde_json::json;
use tokio_postgres::NoTls;
use chrono::SecondsFormat::Micros;
use crate::models::{RespostaTransacao, RequestTransacao, TransacaoRespostaExtrato};

mod models;

#[post("/clientes/{id}/transacoes")]
async fn transacao(
    path: web::Path<i32>,
    transacao: Bytes,
    client: web::Data<Arc<tokio_postgres::Client>>
    ) -> impl Responder {

    let transacao: RequestTransacao = match serde_json::from_slice(&transacao) {
        Ok(tr) => tr,
        Err(_) => return HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY).body("vim é melhor que nano"),
        
   };

    if path.abs() < 1 || path.abs() > 5 {
        return HttpResponse::build(StatusCode::NOT_FOUND)
            .body("cliente não encontrado.");
    }

    if  transacao.descricao.len() > 10 || transacao.descricao.len() == 0{
        return HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY).body("Linux>>>>Windows");
    }

    if transacao.tipo != "c" && transacao.tipo != "d" {
        return HttpResponse::build(StatusCode::UNPROCESSABLE_ENTITY)
            .body("tipo inválido");
    }

    let res = client.query(
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
async fn extrato(path: web::Path<i32>, client: web::Data<Arc<tokio_postgres::Client>>) -> impl Responder {
    if path.abs() < 1 || path.abs() > 5 {
        return HttpResponse::build(StatusCode::NOT_FOUND)
            .body("rapaiz");
    }

    let sql1 = "SELECT saldo, limite FROM clientes WHERE id = $1";

    let cliente = client.query(
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
        LIMIT 10";

    let transacoes = client.query(
        sql2,
        &[&path.abs()])
        .await
        .unwrap();


    let mut v = Vec::new();
    for i in &transacoes {
        let a: DateTime<Utc> = i.get(3);
        let b = a.to_rfc3339_opts(Micros,true);
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
            "data_extrato": Utc::now().to_rfc3339_opts(Micros,true),
            "limite": limite,
            
        },
        "ultimas_transacoes": &v,
    });
    
    HttpResponse::Ok().json(response_body)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let mut c = tokio_postgres::Config::new();
    c.user(&(env::var("POSTGRES_USER").expect("Failed to read POSTGRES_USER env var")));
    c.dbname(&(env::var("POSTGRES_DB").expect("Failed to read POSTGRES_DB env var")));
    c.password(&(env::var("POSTGRES_PASSWORD").expect("Failed to read POSTGRES_PASSWORD env var")));
    c.host(&(env::var("DB_HOSTNAME").expect("Failed to read DB_HOSTNAME env var")));
    let (client, conn) = match c.connect(NoTls).await {
        Ok(t) => t,
        Err(err) => panic!("{}", err),
    };

    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("erro de conexão: {}", e);
        }
    });

    let d = Arc::new(client);
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(d.clone()))
            .service(transacao)
            .service(extrato)
    })
    .bind(("0.0.0.0",8000))?
    .run()
    .await
}
