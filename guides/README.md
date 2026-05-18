# Panduan Pengembangan

Dokumen ini menjelaskan arsitektur kode dan konfigurasi connection pool database untuk **rust-starterkit**.

---

## Arsitektur & struktur kode

Proyek memakai **feature-based layout**: setiap domain bisnis punya modul sendiri di `features/`, sementara hal lintas (error, response, middleware) ada di `common/`.

### Alur request

```
HTTP Request
    → RequestIdMiddleware      (header x-request-id)
    → TracingLogger            (logging HTTP)
    → Route handler            (public atau protected)
    → AuthenticateMiddleware   (hanya route terlindungi)
    → Handler → Service → SeaORM / utils
    → ApiResponse JSON
```

- **Public routes** (`/`, `/auth/login`, `/auth/refresh`) — tanpa token.
- **Protected routes** (`/users/*`, `/auth/logout`) — membutuhkan `Authorization: Bearer <access_token>`.

Routing didefinisikan di `src/main.rs`; logic bisnis ada di `features/*/service.rs`, bukan di handler.

### Pohon direktori

```
src/
├── main.rs                         # Entry point, HttpServer, routing publik/terlindungi
├── lib.rs                          # Re-export modul library
├── config.rs                       # AppConfig dari environment (prefix APP_)
├── state.rs                        # AppState & AppServices (DI sederhana)
│
├── bin/
│   └── generate-paseto-keys.rs     # CLI generate kunci PASETO v4
│
├── common/                         # Shared di seluruh aplikasi
│   ├── error.rs                    # ApiError & mapping ke HTTP
│   ├── response.rs                 # Envelope ApiResponse + Meta
│   ├── extractor.rs                # AuthUser extractor (route terlindungi)
│   ├── pagination.rs               # PaginatedQuery & response paginasi
│   ├── constants.rs                # Default pagination, Argon2, header names
│   └── middleware/
│       ├── authenticate.rs         # Verifikasi Bearer + PASETO claims
│       └── request_id.rs           # Inject/propagate x-request-id
│
├── features/                       # Satu modul per domain
│   ├── auth/
│   │   ├── dto.rs                  # LoginRequest, LoginResponse, dll.
│   │   ├── service.rs              # login, refresh, logout
│   │   ├── handler.rs              # Thin handler → service
│   │   └── routes.rs               # /auth/*
│   ├── user/
│   │   ├── entity.rs               # SeaORM entity (tabel users)
│   │   ├── dto.rs                  # Create/Update request & UserResponse
│   │   ├── service.rs              # CRUD + paginasi
│   │   ├── handler.rs
│   │   └── routes.rs               # /users/*
│   └── health/
│       ├── handler.rs              # GET /
│       └── routes.rs
│
├── infrastructure/
│   ├── database/
│   │   ├── connection.rs           # Pool PostgreSQL (SeaORM ConnectOptions)
│   │   └── migrations/             # Crate migrasi SeaORM terpisah
│   ├── external/                   # Placeholder: klien API pihak ketiga
│   └── repositories/               # Placeholder: lapisan repository (opsional)
│
└── utils/
    ├── paseto.rs                   # Init & create/verify token PASETO v4
    ├── password.rs                 # Hash & verify Argon2
    ├── uuid.rs                     # Helper UUID
    └── duration.rs                 # Parse durasi token (mis. "15m", "7d")
```

### Peran tiap lapisan (feature)

| File | Tanggung jawab |
|------|----------------|
| `routes.rs` | Daftar path & method HTTP |
| `handler.rs` | Ekstrak input HTTP, panggil service, kembalikan `HttpResponse` |
| `service.rs` | Logic bisnis, akses database, error domain |
| `dto.rs` | Struct request/response (Serialize/Deserialize) |
| `entity.rs` | Model SeaORM (hanya jika modul punya tabel sendiri) |

### State & dependency injection

`AppState` (di `state.rs`) menyimpan:

- `db` — koneksi SeaORM
- `config` — `AppConfig`
- `services` — `AuthService`, `UserService` (di-wrap `Arc`)

Handler menerima `web::Data<AppState>`; service tidak perlu tahu tentang Actix Web.

### Menambah fitur baru

1. Buat folder `src/features/<nama>/` dengan `mod.rs`, `routes.rs`, `handler.rs`, `service.rs`, `dto.rs`.
2. Tambahkan `pub mod <nama>;` di `features/mod.rs`.
3. Daftarkan route di `main.rs` (`public_routes` atau `protected_routes`).
4. Jika perlu tabel baru: buat migrasi di `infrastructure/database/migrations/`.
5. Daftarkan service baru di `AppServices` (`state.rs`) bila handler membutuhkannya.

Modul `infrastructure/external` dan `infrastructure/repositories` disiapkan sebagai placeholder untuk integrasi eksternal atau pola repository di masa depan.

---

## Konfigurasi database connection pool

Pool dikonfigurasi lewat environment variable `APP_DATABASE__*` dan diterapkan di `infrastructure/database/connection.rs` via `ConnectOptions` SeaORM/SQLx.

### Arti setiap parameter

| Variabel | Satuan | Fungsi |
|----------|--------|--------|
| `MAX_CONNECTIONS` | koneksi | Batas maksimum koneksi aktif di pool |
| `MIN_CONNECTIONS` | koneksi | Koneksi yang dijaga tetap hidup (warm pool) |
| `CONNECT_TIMEOUT` | detik | Batas waktu membuka koneksi baru ke PostgreSQL |
| `ACQUIRE_TIMEOUT` | detik | Batas waktu menunggu koneksi bebas dari pool |
| `IDLE_TIMEOUT` | detik | Koneksi idle dibuang setelah durasi ini |
| `MAX_LIFETIME` | detik | Umur maksimum satu koneksi sebelum diganti |

**Catatan:** Total `MAX_CONNECTIONS` dari semua instance aplikasi tidak boleh melebihi `max_connections` PostgreSQL (kurangi margin untuk admin/backup/migrasi). Jika menjalankan beberapa replika, kalikan `MAX_CONNECTIONS × jumlah instance`.

### Preset rekomendasi

Salin blok yang sesuai ke `.env` dan sesuaikan dengan beban aktual Anda.

#### 1. App kecil / development / startup

Default proyek — cocok untuk side project, MVP, internal tool, traffic **< 500 req/min**.

```env
APP_DATABASE__MAX_CONNECTIONS=15
APP_DATABASE__MIN_CONNECTIONS=2
APP_DATABASE__CONNECT_TIMEOUT=10
APP_DATABASE__ACQUIRE_TIMEOUT=8
APP_DATABASE__IDLE_TIMEOUT=300
APP_DATABASE__MAX_LIFETIME=1800
```

#### 2. App menengah (rekomendasi umum)

Cocok untuk startup, SaaS kecil–menengah, traffic **500 – 3.000 req/min**.

```env
APP_DATABASE__MAX_CONNECTIONS=40
APP_DATABASE__MIN_CONNECTIONS=6
APP_DATABASE__CONNECT_TIMEOUT=12
APP_DATABASE__ACQUIRE_TIMEOUT=10
APP_DATABASE__IDLE_TIMEOUT=600
APP_DATABASE__MAX_LIFETIME=2700
```

#### 3. App besar / high traffic

Cocok untuk traffic tinggi, banyak user concurrent, atau beberapa microservice yang memakai DB yang sama — traffic **> 3.000 req/min** (sesuaikan dengan kapasitas server DB).

```env
APP_DATABASE__MAX_CONNECTIONS=120
APP_DATABASE__MIN_CONNECTIONS=15
APP_DATABASE__CONNECT_TIMEOUT=15
APP_DATABASE__ACQUIRE_TIMEOUT=12
APP_DATABASE__IDLE_TIMEOUT=900
APP_DATABASE__MAX_LIFETIME=3600
```

### Tips tuning

- Naikkan `ACQUIRE_TIMEOUT` jika log sering menunjukkan timeout saat peak load.
- `MIN_CONNECTIONS` terlalu tinggi membuang resource di idle; terlalu rendah menambah latency saat spike.
- Pantau metrik pool (koneksi aktif, wait time) sebelum naik ke preset berikutnya.
- Setelah mengubah `.env`, restart aplikasi agar pool dibuat ulang dengan nilai baru.
