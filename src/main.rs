use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use rinha24::{*, models::{Cliente, Transacao, NovaTransacao, RequestTransacao, RespostaTransacao},schema::clientes::{self},schema::transacoes::{self}};
use crate::transacoes::*;

use std::env;
use dotenvy::dotenv;

use serde_json::json;

use diesel::prelude::{*, SelectableHelper};
use chrono::{Local,SecondsFormat::Micros};

#[get("/env")]
async fn show_envs() -> impl Responder {

    let db_hostname: String = env::var("ADD_API").expect("Failed to read DB_HOSTNAME env var");
    let postgres_pswd = env::var("POSTGRES_PASSWORD").expect("Failed to read POSTGRES_PASSWORD env var");
    let postgres_user = env::var("POSTGRES_USER").expect("Failed to read POSTGRES_USER env var");
    let postgres_db = env::var("POSTGRES_DB").expect("Failed to read POSTGRES_DB env var");

    // Constrói a resposta com todas as informações das variáveis de ambiente
    let response_body = format!(
        "DB Hostname: {}\nPostgres Password: {}\nPostgres User: {}\nPostgres DB: {}",
        db_hostname, postgres_pswd, postgres_user, postgres_db
    ); 

    HttpResponse::Ok().body(response_body)
}

#[get("/banco")]
async fn banco() -> impl Responder {
    let connection = &mut establish_connection();

    let res = clientes::table.select(Cliente::as_select()).load(connection).expect("Error loading clients");

    let response_body = json!(res);

    HttpResponse::Ok().json(response_body) 
}

#[post("/clientes/{id}/transacoes")]
async fn transacao(path: web::Path<i32>, transacao: web::Json<RequestTransacao>) -> impl Responder {
    let connection = &mut establish_connection();

    let cliente: Result<Option<(i32, i32)>, diesel::result::Error> = clientes::table
        .find(path.abs())
        .select((clientes::limite, clientes::saldo))
        .first(connection)
        .optional();


    let cliente = match cliente {
        Ok(Some(cliente)) => cliente,
        Ok(None) => return HttpResponse::NotFound().body(format!("Id de cliente não cadastrado")),
        Err(_) => return HttpResponse::Ok().body("Errinho"),
    };

    let (limite, saldo) = cliente;

    let mut nova_transacao = NovaTransacao {
        id_cliente: path.abs(),
        valor: transacao.valor,
        tipo: &transacao.tipo,
        descricao: &transacao.descricao,
        realizada_em: Local::now().to_rfc3339_opts(Micros,true),
    };

    if transacao.tipo == "d" {
        nova_transacao.valor = nova_transacao.valor * -1;
    } else if transacao.tipo != "c" {
        return HttpResponse::Ok().body("Transacao inválida.");
    }

    if nova_transacao.valor + saldo < limite * -1 {
        return HttpResponse::build(actix_web::http::StatusCode::UNPROCESSABLE_ENTITY)
            .body("Não há limite o suficiente.");
    }
    let updated_cliente = diesel::update(clientes::table.find(path.abs()))
        .set(clientes::saldo.eq(saldo+nova_transacao.valor))
        .returning(Cliente::as_returning())
        .get_result(connection)
        .unwrap();

    diesel::insert_into(transacoes::table)
        .values(&nova_transacao)
        .returning(Transacao::as_returning())
        .get_result(connection)
        .expect("Error saving new transaction");

    let res = RespostaTransacao {
        limite: updated_cliente.limite,
        saldo: updated_cliente.saldo,
    };

    return HttpResponse::Ok().json(json!(res))
}

#[get("/clientes/{id}/extrato")]
async fn extrato(path: web::Path<i32>) -> impl Responder {
    let connection = &mut establish_connection();
    
    let res_cliente = clientes::table
        .filter(clientes::id.eq(path.abs()))
        .select((clientes::id,clientes::saldo,clientes::limite))
        .load::<(i32,i32,i32)>(connection)
        .expect("Error loading clients");

    if !res_cliente.is_empty(){
        let res_transacoes = transacoes::table
            .filter(transacoes::id_cliente.eq(path.abs()))
            .limit(10)
            .order(transacoes::realizada_em.desc())
            .select((valor,descricao,tipo,realizada_em))
            .load::<(i32,Option<String>,String,String)>(connection)
            .expect("Error loading transactions");

        let response_body = json!({
            "saldo": {
                "limite": res_cliente[0].1,
                "total": res_cliente[0].2,
                "data_extrato":  Local::now().to_rfc3339_opts(Micros,true),
                
            },
            "ultimas_transacoes": &res_transacoes,
        });
        
        return HttpResponse::Ok().json(response_body); 
    }
    HttpResponse::NotFound().body("Erro ao acessar clientes")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    HttpServer::new(|| {
        App::new()
            .service(show_envs)
            .service(banco)
            .service(transacao)
            .service(extrato)
    })
    .bind(("0.0.0.0",8000))?
    .run()
    .await
}
