# kreuzberg-fork-triage

[Kreuzberg 4.8.4](https://github.com/Goldziher/kreuzberg) fork'u **üç additive patch** ile:
bu patch'ler [`kreuzberg-ocr-triage`](https://github.com/hasantr/kreuzberg-ocr-triage)
adapter'ının çalışması için gerekli. Upstream'e PR olarak henüz sunulmadı; bu
fork public olarak kullanılabilir durumda.

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

## Upstream PR durumu

Henüz PR açılmadı. Bu fork **self-hosted** kullanımı için var:

```toml
[dependencies]
kreuzberg = { git = "https://github.com/hasantr/kreuzberg-fork-triage", default-features = false, features = ["pdf", "ocr", "office", "html", "tokio-runtime"] }
```

Upstream Kreuzberg 4.9+ bu patch'leri kabul ederse bu fork arşivlenir ve
tüketiciler vanilla Kreuzberg'e döner.

## Lisans

Upstream Kreuzberg [MIT](LICENSE) lisansı altında. Fork ve patch'ler de MIT.
