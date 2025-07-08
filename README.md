Insert data into PostgreSQL (or other databases that support variant types, but currently only PostgreSQL) via HTTP POST

It can be an object with any fields

Unlike FerretDB, which creates a JSONB field and stores everything inside it
SQL operations are very inconvenient, so it simulates a MongoDB interface

But the MongoDB interface is also quite poor

Often what's truly needed is uploading in SQL style, but extra fields are aggregated into one JSONB (variant)
