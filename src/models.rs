use serde::{Serialize, Deserialize};


#[derive(Serialize)]
pub struct Cliente {
    pub id: i32,
    pub nome: String,
    pub limite: i32,
    pub saldo: i32,
}

#[derive(Serialize)]
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
    pub realizad_em: String,
}

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
