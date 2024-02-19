use std::time::SystemTime;
use diesel::prelude::*;
use serde::Serialize;

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
