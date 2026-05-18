# Panduan Commit Message

Proyek ini mengikuti **[Conventional Commits](https://www.conventionalcommits.org/)** agar riwayat Git mudah dibaca dan selaras dengan [CHANGELOG.md](../CHANGELOG.md).

## Format

```
<type>(<scope>): <deskripsi singkat>

[body opsional]

[footer opsional]
```

### Aturan singkat

| Aturan | Detail |
| ------ | ------ |
| **type** | Wajib. Jenis perubahan (lihat tabel di bawah). |
| **scope** | Opsional. Modul/area yang terdampak (`auth`, `user`, `db`, …). |
| **deskripsi** | Wajib. Imperatif, huruf kecil, tanpa titik di akhir, maks. ~72 karakter. |
| **body** | Opsional. Jelaskan *mengapa* dan *apa dampaknya*, bukan ulang diff. |
| **footer** | Opsional. `BREAKING CHANGE:`, `Closes #123`, co-author, dll. |

### Contoh minimal

```
feat(auth): add refresh token rotation on login
```

### Contoh lengkap

```
fix(user): reject duplicate email on create

Service now checks unique constraint before insert and maps
conflict to 409 Conflict instead of 500.

Closes #42
```

### Breaking change

Tambahkan `!` setelah type/scope **atau** footer `BREAKING CHANGE:`:

```
feat(auth)!: replace JWT with PASETO v4 tokens

BREAKING CHANGE: API clients must send PASETO tokens; JWT is no longer accepted.
```

---

## Type

| Type | CHANGELOG | Kapan dipakai |
| ---- | --------- | ------------- |
| `feat` | Added | Fitur baru |
| `fix` | Fixed | Perbaikan bug |
| `docs` | — | README, guides, komentar docs-only |
| `refactor` | Changed | Restruktur kode tanpa ubah perilaku |
| `perf` | Changed | Peningkatan performa |
| `test` | — | Test baru atau perbaikan test |
| `build` | — | Cargo, Dockerfile, build script |
| `ci` | — | GitHub Actions, pipeline CI |
| `chore` | — | Maintenance (deps bump, .gitignore) |
| `revert` | — | Revert commit sebelumnya |

> Type lain (`style`, `ci`, dll.) boleh dipakai jika tim setuju; yang di atas cukup untuk starter kit ini.

---

## Scope (rekomendasi)

Gunakan scope yang mencerminkan struktur proyek:

| Scope | Area |
| ----- | ---- |
| `auth` | `features/auth` |
| `user` | `features/user` |
| `health` | `features/health` |
| `db` | SeaORM, migrasi, connection pool |
| `middleware` | `common/middleware` |
| `config` | `config.rs`, env |
| `api` | Response envelope, error handling umum |
| `deps` | Pembaruan dependency (`Cargo.toml`) |
| `cli` | Binary `generate-paseto-keys` |

Scope boleh dihilangkan jika perubahan lintas banyak modul:

```
chore: bump actix-web to 4.13.0
```

---

## Deskripsi yang baik vs buruk

| Buruk | Baik |
| ----- | ---- |
| `fixed bug` | `fix(auth): invalidate refresh token on logout` |
| `update user stuff` | `feat(user): add search filter on list endpoint` |
| `WIP` | *(jangan commit WIP ke main; pakai branch draft)* |
| `misc changes` | `refactor(common): extract pagination defaults to constants` |

Gunakan **bahasa Inggris** untuk subject agar konsisten dengan ekosistem Rust/open source (body boleh Indonesia jika tim lokal).

---

## Mapping ke CHANGELOG

Saat menyiapkan rilis, pindahkan entri dari `[Unreleased]` berdasarkan type commit:

| Commit type | Bagian CHANGELOG |
| ----------- | ---------------- |
| `feat` | **Added** |
| `fix` | **Fixed** |
| `perf`, `refactor` | **Changed** |
| deprecation commit | **Deprecated** |
| removal commit | **Removed** |
| `security` / fix CVE | **Security** |

Contoh: 3 commit `feat(auth)`, `fix(user)`, `docs` → di CHANGELOG: Added (auth), Fixed (user); commit `docs` sering tidak perlu masuk CHANGELOG kecuali dokumentasi signifikan.

---

## Contoh untuk proyek ini

```bash
# Fitur
feat(auth): implement login with email and password
feat(user): add paginated list with sort_by and sort_order

# Bug fix
fix(middleware): return 401 when Bearer prefix is missing
fix(db): set connect timeout from APP_DATABASE__CONNECT_TIMEOUT

# Refactor / perf
refactor(auth): move token creation to paseto utils
perf(user): add index hint for email lookup

# Docs & chore
docs: add database pool presets to guides
chore(deps): update sea-orm to 2.0.0-rc

# Breaking
feat(api)!: wrap all responses in ApiResponse envelope
```

---

## Referensi

- [Conventional Commits](https://www.conventionalcommits.org/)
- [Keep a Changelog](../CHANGELOG.md)
- [Semantic Versioning](https://semver.org/)
