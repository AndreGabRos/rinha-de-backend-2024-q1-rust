use diesel::prelude::*;

#[diesel(table_name = crate::schema::clientes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Queryable, Selectable)]
pub struct Cliente {
    id: i32,
    nome: String,
    limite: i32,
    saldo: i32,
}

#[diesel(table_name = crate::schema::transacoes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Queryable, Selectable)]
pub struct Transacao {
    id: i32,
    id_cliente: i32,
    valor: i32,
    tipo: char,
    descricao: Option<String>,
    realizada_em: chrono::NaiveDateTime,
}