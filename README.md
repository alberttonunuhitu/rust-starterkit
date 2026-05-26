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

**Jalankan lokal (cargo):**

- [Rust](https://rustup.rs/) (edition 2024)
- PostgreSQL
- Database `rust_starterkit` (atau sesuaikan URL di `.env`)

**Atau Docker** — lihat [Docker](#docker); cukup Docker Compose, tanpa Rust/PostgreSQL di host (kunci PASETO tetap digenerate sekali via `cargo` atau disalin dari environment lain).

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

## Docker

Alternatif menjalankan seluruh stack (PostgreSQL, migrasi, API) tanpa menginstal Rust/PostgreSQL di host.

### Prasyarat

- [Docker](https://docs.docker.com/get-docker/) & Docker Compose v2

### 1. Environment

```bash
cp .env.example .env
```

Isi minimal di `.env`:

| Variabel | Deskripsi |
| -------- | --------- |
| `POSTGRES_PASSWORD` | Password user PostgreSQL |
| `APP_TOKEN__SECRET_KEY` | Kunci akses PASETO (lihat langkah generate di atas) |
| `APP_TOKEN__REFRESH_SECRET_KEY` | Kunci refresh PASETO |

Opsional:

| Variabel | Default | Deskripsi |
| -------- | ------- | --------- |
| `POSTGRES_USER` | `postgres` | User database |
| `POSTGRES_DB` | `rust_starterkit` | Nama database |
| `API_HOST` | `127.0.0.1` | Host bind port API di mesin Anda |
| `API_PORT` | `8080` | Port API di host |
| `RUST_LOG` | `info` | Level log (`debug`, `api=debug`, dll.) |
| `APP_APP__ENVIRONMENT` | `production` (di container) | Override lewat `.env` jika perlu |

Kunci PASETO tetap digenerate di host (sekali):

```bash
cargo run --bin generate-paseto-keys
```

### 2. Build & jalankan

Dari root proyek:

```bash
docker compose --profile dev up --build -d
```

Urutan layanan:

1. **postgres** — menunggu healthcheck `pg_isready`
2. **migrate** — menjalankan migrasi SeaORM (`up`), lalu selesai
3. **api** — server Actix Web di port `8080` dalam container

Cek status:

```bash
docker compose --profile dev ps
curl http://127.0.0.1:8080/
```

### 3. Perintah berguna

```bash
# Log API
docker compose --profile dev logs -f api

# Hentikan stack (data Postgres tetap di volume)
docker compose --profile dev down

# Hentikan + hapus volume database
docker compose --profile dev down -v

# Rebuild image setelah ubah kode
docker compose --profile dev up --build -d
```

### Build image saja

```bash
docker build --target api -t rust-starterkit-api .
docker build --target migrate -t rust-starterkit-migrate .
```

Image API memakai stage multi-stage (`cargo-chef`); env runtime di-inject oleh Compose, bukan file `.env` di dalam image.

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
- Level: atur lewat `RUST_LOG` (default `info`, contoh: `RUST_LOG=debug` atau `RUST_LOG=api=debug,actix_web=info`)

## Catatan pengembangan

- **User pertama**: endpoint `POST /users` terlindungi autentikasi. Untuk bootstrap, buat user pertama lewat SQL/migrasi seed, atau tambahkan route registrasi publik sementara.
- **Logout** mengembalikan `204 No Content` tanpa body JSON.
- Edition Rust: **2024** (`Cargo.toml`).
