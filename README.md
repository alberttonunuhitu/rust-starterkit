# rust-starterkit

Starter kit REST API dengan **Rust**, **Actix Web**, **PostgreSQL** (SeaORM), autentikasi **PASETO v4** (Ed25519), dan hashing password **Argon2**.

## Fitur

- Arsitektur berbasis fitur (`auth`, `user`, `health`)
- Token akses & refresh (PASETO v4) dengan rotasi refresh token
- CRUD pengguna dengan paginasi, pencarian, dan sorting
- Middleware: `x-request-id`, autentikasi Bearer, tracing HTTP
- Logging terstruktur (JSON ke file harian + pretty console)
- Konfigurasi via environment (`APP_*`) dan `.env`
- Migrasi database SeaORM

## Tech stack

| Layer    | Teknologi                     |
| -------- | ----------------------------- |
| HTTP     | Actix Web 4                   |
| ORM      | SeaORM 2 (PostgreSQL)         |
| Auth     | pasetors (PASETO v4, Ed25519) |
| Password | Argon2                        |
| Config   | `config` + `dotenvy`          |
| Logging  | `tracing`, `tracing-actix-web`|

## Prasyarat

- [Rust](https://rustup.rs/) (edition 2024)
- PostgreSQL
- Database `rust_starterkit` (atau sesuaikan URL di `.env`)

## Dokumentasi lanjutan

| Dokumen | Isi |
| ------- | --- |
| [guides/README.md](guides/README.md) | Arsitektur, struktur kode, connection pool |
| [guides/COMMITS.md](guides/COMMITS.md) | Rekomendasi format commit message Git |
| [CHANGELOG.md](CHANGELOG.md) | Riwayat perubahan versi |

## Memulai

### 1. Clone & salin environment

```bash
cp .env.example .env
```

Sesuaikan `APP_DATABASE__URL` dan pengaturan server di `.env`.

### 2. Generate kunci PASETO

```bash
cargo run --bin generate-paseto-keys
```

Salin output ke `.env`:

- `APP_TOKEN__SECRET_KEY`
- `APP_TOKEN__REFRESH_SECRET_KEY`

Opsi format hex:

```bash
cargo run --bin generate-paseto-keys -- --hex
```

### 3. Migrasi database

```bash
cd src/infrastructure/database/migrations
cargo run -- up
cd ../../../../..
```

Pastikan `DATABASE_URL` di `migrations/.env` mengarah ke database yang sama dengan `APP_DATABASE__URL`.

### 4. Jalankan server

```bash
cargo run
```

Server default: `http://127.0.0.1:8080`

## Konfigurasi

Variabel memakai prefix `APP_`, nested dengan `__` (contoh: `APP_SERVER__PORT`).

| Variabel                          | Deskripsi              | Default                                              |
| --------------------------------- | ---------------------- | ---------------------------------------------------- |
| `APP_SERVER__HOST`                | Bind host              | `127.0.0.1`                                          |
| `APP_SERVER__PORT`                | Port                   | `8080`                                               |
| `APP_DATABASE__URL`               | PostgreSQL URL         | `postgres://postgres@localhost:5432/rust_starterkit` |
| `APP_TOKEN__SECRET_KEY`           | Kunci akses PASETO     | —                                                    |
| `APP_TOKEN__REFRESH_SECRET_KEY`   | Kunci refresh PASETO   | —                                                    |
| `APP_TOKEN__EXPIRATION`           | TTL access token       | `15m`                                                |
| `APP_TOKEN__REFRESH_EXPIRATION`   | TTL refresh token      | `7d`                                                 |
| `APP_TOKEN__ISSUER`               | Issuer token           | `rust`                                               |
| `APP_TOKEN__AUDIENCE`             | Audience token         | `vue`                                                |
| `APP_APP__ENVIRONMENT`            | Environment aplikasi   | `development`                                        |

Lihat `.env.example` untuk daftar lengkap.

## API

Semua respons sukses (kecuali logout) memakai envelope:

```json
{
  "success": true,
  "data": { ... },
  "meta": {
    "request_id": "...",
    "timestamp": "..."
  }
}
```

Route terlindungi membutuhkan header:

```
Authorization: Bearer <access_token>
```

### Publik

| Method | Path            | Deskripsi                              |
| ------ | --------------- | -------------------------------------- |
| `GET`  | `/`             | Status aplikasi                        |
| `POST` | `/auth/login`   | Login (`email`, `password`)            |
| `POST` | `/auth/refresh` | Refresh token (`refresh_token`)        |

### Terlindungi (Bearer)

| Method   | Path           | Deskripsi                            |
| -------- | -------------- | ------------------------------------ |
| `POST`   | `/auth/logout` | Logout (invalidasi refresh token)    |
| `GET`    | `/users`       | Daftar user (paginasi)               |
| `POST`   | `/users`       | Buat user                            |
| `GET`    | `/users/{id}`  | Detail user                          |
| `PUT`    | `/users/{id}`  | Update user                          |
| `DELETE` | `/users/{id}`  | Hapus user                           |

### Contoh

**Login**

```bash
curl -X POST http://127.0.0.1:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"secret"}'
```

**Daftar users (paginasi)**

```bash
curl "http://127.0.0.1:8080/users?page=1&per_page=10" \
  -H "Authorization: Bearer <access_token>"
```

Query paginasi: `page`, `per_page`, `search`, `sort_by`, `sort_order`.

## Logging
- Output: **stdout** (satu baris JSON per event, RFC 3339, span untuk request HTTP)
- Level: atur lewat `RUST_LOG` (default `info`, contoh: `RUST_LOG=debug` atau `RUST_LOG=rust_starterkit=debug,actix_web=info`)

## Catatan pengembangan

- **User pertama**: endpoint `POST /users` terlindungi autentikasi. Untuk bootstrap, buat user pertama lewat SQL/migrasi seed, atau tambahkan route registrasi publik sementara.
- **Logout** mengembalikan `204 No Content` tanpa body JSON.
- Edition Rust: **2024** (`Cargo.toml`).
