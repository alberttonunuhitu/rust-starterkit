# Changelog

Semua perubahan penting pada proyek ini didokumentasikan di file ini.

Format mengikuti [Keep a Changelog](https://keepachangelog.com/id/1.1.0/),
dan proyek ini mematuhi [Semantic Versioning](https://semver.org/lang/id/).

## Cara menulis entri

Setiap versi memakai tanggal rilis (`YYYY-MM-DD`) dan dikelompokkan ke kategori berikut (kosongkan kategori yang tidak dipakai):

| Kategori | Kapan dipakai |
| -------- | ------------- |
| **Added** | Fitur baru |
| **Changed** | Perubahan pada fitur yang sudah ada |
| **Deprecated** | Fitur yang akan dihapus di versi mendatang |
| **Removed** | Fitur yang dihapus |
| **Fixed** | Perbaikan bug |
| **Security** | Perbaikan kerentanan keamanan |

Contoh entri:

```markdown
## [0.2.0] - 2026-06-01

### Added
- Endpoint registrasi publik `POST /auth/register`

### Fixed
- Validasi email case-insensitive pada login
```

Perubahan yang belum dirilis tulis di bagian `[Unreleased]`. Saat rilis, pindahkan isinya ke versi baru dan kosongkan `[Unreleased]`.

Format commit message mengikuti [guides/COMMITS.md](guides/COMMITS.md) (Conventional Commits).

---

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

---

## [0.1.0] - 2026-05-18

Rilis awal **rust-starterkit**.

### Added

- REST API dengan Actix Web 4
- Autentikasi PASETO v4 (access & refresh token) dan logout
- CRUD pengguna dengan paginasi, pencarian, dan sorting
- PostgreSQL via SeaORM 2 dan migrasi (`users`, trigger `updated_at`)
- Hashing password Argon2
- Middleware `x-request-id` dan autentikasi Bearer
- Logging terstruktur (JSON ke file + console)
- Konfigurasi environment (`APP_*`) dan `.env.example`
- Binary `generate-paseto-keys` untuk kunci PASETO
- Dokumentasi `README.md` dan `guides/README.md`

[Unreleased]: https://github.com/YOUR_USER/rust-starterkit/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/YOUR_USER/rust-starterkit/releases/tag/v0.1.0
