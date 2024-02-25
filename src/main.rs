use actix_web::{get, post, web::{self, Data}, App, HttpResponse, HttpServer, Responder, http};
use deadpool_postgres::{Runtime, GenericClient};
use rinha24::{models::{NovaTransacao, RequestTransacao, RespostaTransacao, TransacaoRespostaExtrato}};
use dotenvy::dotenv;
use serde_json::json;
use tokio_postgres::{NoTls};
use chrono::{Local,SecondsFormat::Micros};


#[post("/clientes/{id}/transacoes")]
async fn transacao(
    path: web::Path<i32>,
    transacao: web::Json<RequestTransacao>,
    connection: web::Data<deadpool_postgres::Pool>
) -> impl Responder {
    let connection = connection.get().await.expect("error connecting to postgres");

    if transacao.tipo != "d" && transacao.tipo != "c" {
        return HttpResponse::build(http::StatusCode::UNPROCESSABLE_ENTITY).body("tipo de transação inválido");
    }

    let cliente = connection.query(
        "SELECT saldo, limite FROM clientes WHERE id = $1", 
        &[&path.abs()])
        .await
        .unwrap();

    let row = &cliente[0];
    let saldo: i32 = row.get(0);
    let limite: i32 = row.get(1);

    let nova_transacao = NovaTransacao {
        id_cliente: path.abs(),
        valor: if transacao.tipo == "d" {
            transacao.valor * -1
        } else {
            transacao.valor
        },
        tipo: &transacao.tipo,
        descricao: &transacao.descricao,
        // realizada_em: Local::now().to_rfc3339_opts(Micros,true),
        realizada_em: "123".to_string()
    };

    if nova_transacao.valor + saldo < limite * -1 {
        return HttpResponse::build(actix_web::http::StatusCode::UNPROCESSABLE_ENTITY)
            .body("não há limite o suficiente.");
    }

    let novo_saldo = saldo + nova_transacao.valor;

    connection.query(
        "INSERT INTO transacoes (id_cliente, valor, tipo, descricao, realizada_em) VALUES ($1, $2, $3, $4, $5)",
        &[&nova_transacao.id_cliente, &nova_transacao.valor, &nova_transacao.tipo, &nova_transacao.descricao, &nova_transacao.realizada_em])
        .await
        .unwrap();
    
    connection.query(
        "UPDATE clientes SET saldo = $1 WHERE id = $2",
        &[&novo_saldo, &path.abs()]
    )
        .await
        .unwrap();

    // let mut nova_transacao = NovaTransacao {
    //     id_cliente: path.abs(),
    //     valor: transacao.valor,
    //     tipo: &transacao.tipo,
    //     descricao: &transacao.descricao,
    //     realizada_em: Local::now().to_rfc3339_opts(Micros,true),
    // };

    // if transacao.tipo == "d" {
    //     nova_transacao.valor = nova_transacao.valor * -1;
    // } else if transacao.tipo != "c" {
    //     return HttpResponse::Ok().body("Transacao inválida.");
    // }

    // let updated_cliente = diesel::update(clientes::table.find(path.abs()))
    //     .set(clientes::saldo.eq(saldo+nova_transacao.valor))
    //     .returning(Cliente::as_returning())
    //     .get_result(&mut connection.conn)
    //     .unwrap();

    // diesel::insert_into(transacoes::table)
    //     .values(&nova_transacao)
    //     .returning(Transacao::as_returning())
    //     .get_result(&mut connection.conn)
    //     .expect("Error saving new transaction");

    // let res = RespostaTransacao {
    //     limite: updated_cliente.limite,
    //     saldo: updated_cliente.saldo,
    // };

    // return HttpResponse::Ok().body("oi")

    let resposta = RespostaTransacao {
        saldo: novo_saldo,
        limite,
    };

    HttpResponse::Ok().json(json!(resposta))}

#[get("/clientes/{id}/extrato")]
async fn extrato(path: web::Path<i32>, connection: web::Data<deadpool_postgres::Pool>
) -> impl Responder {
    if path.abs() > 5 {
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

    // let res_cliente = clientes::table
    //     .filter(clientes::id.eq(path.abs()))
    //     .select((clientes::id,clientes::saldo,clientes::limite))
    //     .load::<(i32,i32,i32)>(connection)
    //     .expect("Error loading clients");

    // if !res_cliente.is_empty(){
    //     let res_transacoes = transacoes::table
    //         .filter(transacoes::id_cliente.eq(path.abs()))
    //         .limit(10)
    //         .order(transacoes::realizada_em.desc())
    //         .select((valor,descricao,tipo,realizada_em))
    //         .load::<(i32,Option<String>,String,String)>(connection)
    //         .expect("Error loading transactions");

    //     let response_body = json!({
    //         "saldo": {
    //             "total": saldo,
    //             "data_extrato":  Local::now().to_rfc3339_opts(Micros,true),
    //             "limite": limite,
    //             
    //         },
    //         "ultimas_transacoes": &res_transacoes,
    //     });
    //     
    //     return HttpResponse::Ok().json(response_body); 
    // }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut pg_config = deadpool_postgres::Config::new();
    pg_config.user = Some("admin".to_string());
    pg_config.host = Some("db".to_string());
    pg_config.password = Some("123".to_string());
    pg_config.dbname = Some("rinha".to_string());

    pg_config.pool = deadpool_postgres::PoolConfig::new(100).into();

    let pg_pool = pg_config.create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("error creating pool");

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
