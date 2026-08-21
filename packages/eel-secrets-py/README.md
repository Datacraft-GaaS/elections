# eel-secrets-py

Async OpenBao (Vault-compatible) secrets client for the eel platform.

## Features

- AppRole authentication with lazy init and token renewal
- KV v2 read/write
- Transit encryption/decryption/key rotation
- Dynamic PostgreSQL credentials with lease renewal and revocation

## Usage

```python
from eel_secrets import OpenBaoClient, Transit, PgDynamicCred

client = OpenBaoClient(
    "https://bao.example.com",
    role_id="...",
    secret_id="...",
)

# KV v2
data = await client.read_secret("secret/myapp")

# Transit
transit = Transit(client)
ciphertext = await transit.encrypt("mykey", b"plaintext")
plaintext = await transit.decrypt("mykey", ciphertext)

# Dynamic PG creds
pg = PgDynamicCred(client, db_role="myapp-role")
cred = await pg.get()
dsn = pg.connection_string(cred, host="db.example.com", port=5432, dbname="mydb")
```
