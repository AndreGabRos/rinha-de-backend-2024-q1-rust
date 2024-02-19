DELETE FROM clientes WHERE
  id IN (SELECT id FROM clientes);
