# kreuzberg-fork-triage

[Kreuzberg 4.9.2](https://github.com/kreuzberg-dev/kreuzberg) fork'u **üç
additive patch** ile. Bu patch'ler
[`kreuzberg-ocr-triage`](https://github.com/hasantr/kreuzberg-ocr-triage)
adapter'ının çalışması için gerekli. Upstream'e PR olarak henüz sunulmadı; bu
fork public olarak kullanılabilir durumda.

> **Not:** Upstream repository 2026-04'te `Goldziher/kreuzberg` →
> `kreuzberg-dev/kreuzberg`'ye taşındı. Bu fork yeni org'un v4.9.2 etiketine
> dayanıyor (~2026-04-19). Requires **Rust 1.95+** (upstream gereksinimi).

## Patch listesi

### 1. `OcrBackend::process_image_raw(rgb, w, h, config)` — raw pixel yolu

Dosya: `src/plugins/ocr.rs`

Mevcut `process_image(bytes, config)` API'sine ek olarak raw RGB8 buffer alan
bir trait metodu. Default implementation bytes'a PNG encode edip `process_image`'a
düşüyor (backward-compat). Ama triage + PDF renderer raw-pixel ile doğrudan
çalışabiliyor, bu sayede PNG encode maliyeti atlanıyor.

### 2. `src/extraction/image_ocr.rs` registry-aware

Mevcut kod sadece Tesseract'ı çağırıyordu. Bu patch, `OcrConfig.backend`
alanına bakıp Kreuzberg OCR backend registry'sinden kaydedilmiş backend'i
kullanıyor. Böylece DOCX/PPTX/Jupyter/Markdown embedded image'ları için de
custom `triage` backend (veya Paddle/Rapid) devreye girebiliyor.

### 3. PDF extractor raw-RGB path

Dosya: `src/extractors/pdf/ocr.rs`

PDF sayfasını pdfium ile RGB buffer'ına render ettikten sonra, mevcut kod
bu buffer'ı PNG'ye encode edip `process_image(bytes)` çağırıyordu (50-200 ms/page
boşa gidiyordu). Bu patch PNG encode aşamasını bypass'ler ve `process_image_raw`
kullanır. Kreuzberg Cargo.toml'a ek olarak `[dependencies.png] = "0.18"`
eklenmiştir (zaten transitive olarak vardı, artık explicit).

## Sürüm ve upgrade notları

- **v4.9.2** baseline (2026-04-19, `kreuzberg-dev/kreuzberg` org).
- Önceki baseline v4.8.4 (2026-04-13); 8 günlük gap'te bug fix'ler +
  LLM usage tracking + smart document chunking eklendi.
- Üç patch'in site'ları değişmedi; minor offset (5-7 satır) ile clean apply.

## Upstream PR durumu

Henüz PR açılmadı. Bu fork **self-hosted** kullanımı için var:

```toml
[dependencies]
kreuzberg = { git = "https://github.com/hasantr/kreuzberg-fork-triage", default-features = false, features = ["pdf", "ocr", "office", "html", "tokio-runtime"] }
```

Upstream Kreuzberg bu patch'leri kabul ederse bu fork arşivlenir ve
tüketiciler vanilla Kreuzberg'e döner.

## Lisans

Upstream Kreuzberg lisansı (Elastic-2.0). Fork ve patch'ler de aynı lisans altında.
