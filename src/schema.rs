// @generated automatically by Diesel CLI.

diesel::table! {
    clientes (id) {
        id -> Int4,
        #[max_length = 100]
        nome -> Varchar,
        limite -> Int4,
        saldo -> Int4,
    }
}

diesel::table! {
    transacoes (id) {
        id -> Int4,
        id_cliente -> Int4,
        valor -> Int4,
        #[max_length = 1]
        tipo -> Bpchar,
        descricao -> Nullable<Text>,
        realizada_em -> Timestamp,
    }
}

diesel::joinable!(transacoes -> clientes (id_cliente));

diesel::allow_tables_to_appear_in_same_query!(
    clientes,
    transacoes,
);
