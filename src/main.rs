use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use rinha24::{*, schema::clientes, models::Cliente};
use std::env;
use diesel::prelude::{*, SelectableHelper};


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

    let mut response_body = String::new(); 

    for cliente in res {
        response_body
            .push_str(&format!("ID: {}, Nome: {}, Limite: {}, Saldo: {}\n",
                                        cliente.id, cliente.nome, cliente.limite, cliente.saldo));
    }

    HttpResponse::Ok().body(response_body) 
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(show_envs)
            .service(banco)
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}
