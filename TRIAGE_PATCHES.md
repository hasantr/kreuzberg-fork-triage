# kreuzberg-fork-triage

[Kreuzberg 4.9.7](https://github.com/kreuzberg-dev/kreuzberg) fork'u **üç
additive triage patch + iki UTF-8 safety patch** ile. Triage patch'ler
[`kreuzberg-ocr-triage`](https://github.com/hasantr/kreuzberg-ocr-triage)
adapter'ının çalışması için gerekli. UTF-8 safety patch'leri Türkçe/Arapça/CJK
gibi multi-byte UTF-8 karakter içeren dökümanlarda byte-slice panic'lerini
önler. Upstream'e PR olarak henüz sunulmadı; bu fork public olarak kullanılabilir
durumda.

> **Not:** Upstream repository 2026-04'te `Goldziher/kreuzberg` →
> `kreuzberg-dev/kreuzberg`'ye taşındı. Bu fork v4.9.7 etiketine
> dayanıyor (~2026-05-08). Requires **Rust 1.95+** (upstream gereksinimi).

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

### 4. EPUB byte-slicing UTF-8 safety

Dosya: `src/extractors/epub/mod.rs`

`collect_annotation_uris` link annotation çıkarırken `&text[ann.start..ann.end]`
raw byte slicing yapıyordu. `ann.start` / `ann.end` byte offset'leri
multi-byte UTF-8 karakter (Türkçe `ı`/`ş`/`ü`, Arapça, CJK, vb.) ortasından
geçtiğinde Rust panic ediyordu — Türkçe epub'lar 1231/1497 dosya scope'unda
sıfır indexing ile sonuçlanıyordu.

Fix: `text.get(start..end).filter(|s| !s.is_empty()).map(|s| s.to_string())`.
Char-boundary mismatch'inde slient `None` döner, link `label: None` ile push
edilir, indexing devam eder.

### 5. Cross-extractor UTF-8 safety audit

Patch 4 çıkışında kreuzberg-fork ve upstream src/extractors/ üzerinde
`&text[..]`, `&body[..]`, `&content[..]` benzeri raw byte slicing pattern'i
audit edildi. Aynı sınıf bug'ı barındıran 3 ek dosya tespit edildi:

- `src/extractors/html.rs` — annotation link label extraction (epub ile identical)
- `src/extractors/jats/mod.rs` — JATS XML inline element span trimming
- `src/extractors/docbook.rs` — DocBook inline element span trimming (jats ile identical)

Hepsine `text.get(start..end)` pattern'i uygulandı. RTF span'ları (8 hit),
email word boundaries (1 hit), ve internal pipeline byte_offset slicing
(transform/, pdf metadata — 5 hit) **kapsam dışı bırakıldı** — bu patch
extractor-side annotation/span pattern'iyle sınırlı tutuldu.

## Sürüm ve upgrade notları

- **v4.9.7** baseline (2026-05-08, `kreuzberg-dev/kreuzberg` org).
- Önceki baseline v4.9.2 (2026-04-19); v4.9.2→v4.9.7 arası 70 commit, çoğu
  fix: PDF tagged-block + image OOM cap + ocr_elements propagation +
  PST attachments + HWP MIME + email HTML fallback + chunking semantic.
- Triage patch'lerinin uygulandığı 3 dosyadan ikisi (image_ocr.rs, pdf/ocr.rs)
  v4.9.7'de upstream tarafından da değiştirildi → conflict. Patch sürümü
  korundu (registry-aware + raw-RGB davranışı patch'in zaten yeniden yazdığı
  bölgeleri kapsıyor).
- Üç patch'in core site'ları değişmedi; minor offset ile clean apply.
- Audit patch (#5) v4.9.7'de yeni eklendi.

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
