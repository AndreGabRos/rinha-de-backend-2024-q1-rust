use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::schema::transacoes;

#[derive(Serialize)]
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::clientes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Cliente {
    pub id: i32,
    pub nome: String,
    pub limite: i32,
    pub saldo: i32,
}

#[derive(Serialize)]
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transacoes)]
#[diesel(check_for_backend(diesel::pg::Pg))]

pub struct Transacao {
    pub id: i32,
    pub id_cliente: i32,
    pub valor: i32,
    pub tipo: String,
    pub descricao: Option<String>,
    pub realizada_em: String,
}

#[derive(Deserialize)]
pub struct RequestTransacao {
    #[serde(deserialize_with = "deserialize_int_or_float")]
    pub valor: i32,
    pub tipo: String,
    pub descricao: String,
}

#[derive(Insertable)]
#[diesel(table_name = transacoes)]
pub struct NovaTransacao<'a> {
    pub id_cliente: i32,
    pub valor: i32,
    pub tipo: &'a str,
    pub descricao: &'a str,
    pub realizada_em: String,
}

#[derive(Serialize)]
pub struct RespostaTransacao {
    pub limite: i32,
    pub saldo: i32,
}

#[derive(Serialize)]
pub struct TransacaoRespostaExtrato<'a> {
    pub valor: i32,
    pub tipo: &'a str,
    pub descricao: &'a str,
    pub realizad_em: &'a str,
}

use actix_web::{error::ErrorBadRequest, web, App, HttpResponse, HttpServer, FromRequest, http};

fn deserialize_int_or_float<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    if let serde_json::Value::Number(num) = &value {
        if let Some(int_value) = num.as_i64() {
            return Ok(int_value as i32);
        }
    }
    Err(serde::de::Error::custom("Invalid value. Expected integer."))
}

// Implement FromRequest trait for RequestTransacao
// impl FromRequest for RequestTransacao {
//     type Error = actix_web::Error;
//     type Future = futures::future::Ready<Result<Self, Self::Error>>;

//     fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
//         let json = web::Json::<Self>::from_request(req, _payload);
//         Box::pin(async move {
//             match json.await {
//                 Ok(payload) => Ok(payload.into_inner()),
//                 Err(_) => return HttpResponse::build(http::StatusCode::UNPROCESSABLE_ENTITY).body("erro"),
//             }
//         })
//     }
// }

// async fn index(payload: RequestTransacao) -> HttpResponse {
//     HttpResponse::Ok().json(payload)
// }

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     HttpServer::new(|| {
//         App::new()
//             .service(web::resource("/").route(web::post().to(index)))
//     })
//     .bind("127.0.0.1:8080")?
//     .run()
//     .await
// }

