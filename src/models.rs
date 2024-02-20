use std::time::SystemTime;
use diesel::{prelude::*, serialize};
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

#[diesel(table_name = crate::schema::transacoes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Queryable, Selectable)]
pub struct Transacao {
    id: i32,
    id_cliente: i32,
    valor: i32,
    tipo: String,
    descricao: Option<String>,
    realizada_em: SystemTime,
}

#[derive(Deserialize)]
pub struct RequestTransacao {
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
    pub realizada_em: SystemTime,
}

#[derive(Serialize)]
pub struct RespostaTransacao {
    pub limite: i32,
    pub saldo: i32,
}
