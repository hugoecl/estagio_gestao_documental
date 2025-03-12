## Setup the database

After installing MySql/MariaDB create the database:

```sql
CREATE DATABASE gestao_documental;
```

The backend expects the root user to have a password of "root"

## Tls certificate

For testing purposes, use [mkcert](https://github.com/FiloSottile/mkcert):

```bash
mkdir certs
cd certs
mkcert -install
mkcert -key-file key.pem -cert-file cert.pem 127.0.0.1 localhost
```
