CREATE TABLE clientes (
  id SERIAL PRIMARY KEY NOT NULL,
  nome VARCHAR(23) NOT NULL, 
  limite INTEGER NOT NULL CHECK (limite >= 0),
  saldo INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE transacoes (
  id SERIAL PRIMARY KEY NOT NULL,
  id_cliente INTEGER NOT NULL,
  valor INTEGER NOT NULL,
  tipo CHAR(1) NOT NULL,
  descricao VARCHAR(10),
  realizada_em TIMESTAMP NOT NULL,

  CONSTRAINT clientes FOREIGN KEY (id_cliente) REFERENCES clientes(id)
);

CREATE PROCEDURE fazer_transacao (
  t_id_cliente INTEGER,
  t_valor INTEGER,
  t_tipo TEXT,
  t_descricao TEXT
)
LANGUAGE plpgsql
AS $$
DECLARE 
c_saldo INTEGER;
c_limite INTEGER;
BEGIN
  SELECT saldo,limite INTO c_saldo,c_limite FROM clientes WHERE id = t_id_cliente;
  -- IF c_saldo - t_valor <= 0 - c_limite THEN
  UPDATE clientes SET saldo = c_saldo + t_valor WHERE id = t_id_cliente;
  INSERT INTO transacoes (id_cliente, valor, tipo, descricao, realizada_em) VALUES (t_id_cliente, t_valor, t_tipo, t_descricao, CURRENT_TIMESTAMP);
  -- ELSE
  --   RAISE EXCEPTION 'transação ultrapassa o limite disponível';
  -- END IF;
END;
$$;

DO $$
BEGIN
  INSERT INTO clientes (nome, limite)
  VALUES
    ('o barato sai caro', 1000 * 100),
    ('zan corp ltda', 800 * 100),
    ('les cruders', 10000 * 100),
    ('padaria joia de cocaia', 100000 * 100),
    ('kid mais', 5000 * 100);
END; $$

-- CALL fazer_transacao (1, 1000, 'c', 'teste');
