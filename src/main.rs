use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use chrono_tz::Tz;
use rinha24::{*, models::{Cliente, Transacao, NovaTransacao, RequestTransacao, RespostaTransacao}};
use rinha24::schema::clientes::{self, limite};
use rinha24::schema::transacoes::{self, id_cliente, descricao};
use serde::Deserialize;
use serde_json::json;
use std::{env, str::FromStr, time::SystemTime};
use diesel::prelude::{*, SelectableHelper};
use chrono::{Local, DateTime};
use chrono::SecondsFormat::Micros;

#[get("/env")]
async fn show_envs() -> impl Responder {

    let db_hostname: String = env::var("DB_HOSTNAME").expect("Failed to read DB_HOSTNAME env var");
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

    let data_atual = Local::now().to_rfc3339_opts(Micros,true);
    let data_atual_convertida = DateTime::parse_from_rfc3339(&data_atual).unwrap();

    let data_atual_convertida_de_novo: SystemTime = data_atual_convertida.into();

    let cliente = clientes::table
        .find(path.abs())
        .select(Cliente::as_select())
        .first(connection)
        .optional();

    let cliente = match cliente {
        Ok(Some(cliente)) => cliente,
        Ok(None) => return HttpResponse::NotFound().body(format!("Id inválido")),
        Err(_) => return HttpResponse::Ok().body("Errinho"),
    };

    let mut nova_transacao = NovaTransacao {
        id_cliente: path.abs(),
        valor: transacao.valor,
        tipo: &transacao.tipo,
        descricao: &transacao.descricao,
        realizada_em: data_atual_convertida_de_novo,
    };

    if transacao.tipo == "d" {
        nova_transacao.valor = nova_transacao.valor * -1;
    } else if transacao.tipo != "c" {
        return HttpResponse::Ok().body("Transacao inválida.");
    }

    if nova_transacao.valor + cliente.saldo < cliente.limite * -1 {
        return HttpResponse::build(actix_web::http::StatusCode::UNPROCESSABLE_ENTITY)
            .body("Não há limite o suficiente.");
    }
    let updated_cliente = diesel::update(clientes::table.find(path.abs()))
        .set(clientes::saldo.eq(cliente.saldo+nova_transacao.valor))
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
        .select(Cliente::as_select())
        .load(connection)
        .expect("Error loading clients");

    if !res_cliente.is_empty(){
        let res_transacoes = transacoes::table
        .filter(transacoes::id_cliente.eq(path.abs()))
        .limit(2)
        .order(transacoes::realizada_em.desc())
        .select(Transacao::as_select())
        .load(connection)
        .expect("Error loading transactions");

        let response_body = json!({
            "saldo": {
                "limite": res_cliente[0].limite,
                "total": res_cliente[0].saldo,
                "data_extrato":  Local::now().to_rfc3339_opts(Micros,true).to_string(),
                
            },
            "ultimas_transacoes": &res_transacoes,
        });
        
        return HttpResponse::Ok().json(response_body); 
    }
    HttpResponse::NotFound().body("Erro ao acessar clientes")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(show_envs)
            .service(banco)
            .service(transacao)
            .service(extrato)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
