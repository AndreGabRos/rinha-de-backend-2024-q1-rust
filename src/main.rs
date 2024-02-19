use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::env;
use rinha24::*;
use diesel::query_dsl::QueryDsl;
use diesel::SelectableHelper;
use diesel::RunQueryDsl;

// This struct represents state
struct AppState {
    app_name: String,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

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
    use self::models::Cliente;
    use rinha24::schema::clientes::dsl::clientes;
    let connection = &mut establish_connection();

    let res  = clientes
        .select(Cliente::as_select())
        .load(connection)
        .expect("Error loading posts");

    let mut response_body = String::new(); 

    for cliente in res {
        response_body.push_str(&format!("ID: {}, Nome: {}, Limite: {}, Saldo: {}\n",
                                        cliente.id, cliente.nome, cliente.limite, cliente.saldo));
    }

    HttpResponse::Ok().body(response_body) 
}


async fn index(data: web::Data<AppState>) -> String {
    let app_name = &data.app_name; // <- get app_name
    format!("Hello {app_name}!") // <- response with app_name
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()

            //Informações do app
            .app_data(web::Data::new(AppState {
                app_name: String::from("Actix Web"),
            }))

            .service(hello)
            .service(echo)
            .service(show_envs)
            .service(banco)
            .service(web::scope("/app")
                // ...so this handles requests for `GET /app/index.html`
                .route("/index.html", web::get().to(index)),)
            .route("/hey", web::get().to(manual_hello))

    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}