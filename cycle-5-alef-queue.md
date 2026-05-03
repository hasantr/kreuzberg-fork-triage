# Cycle 5 Alef Queue — Go E2E Test Failures

Date: 2026-05-03
Regenerated with: alef v0.14.3
Target: Drive Go e2e test suite to 100% green

## Bucket A — Alef Codegen Bugs

Issues requiring alef upstream fixes or regen.

1. **batch_test.go & cache_operations_test.go: missing fixtures and malformed stubs**
   - Alef generates test stubs for categories (batch, cache_operations) that lack fixture files
   - batch_test.go: line 20 unmarshal to `[]string` instead of `[]BatchBytesItem`; line 23 passes wrong type to function
   - batch_test.go: lines 31, 39, 47, 55, 63 pass `nil` to `ExtractFile()` expecting string path
   - cache_operations_test.go: line 20, 29, 37, 45 pass `nil` path to `ExtractFile()`
   - cache_operations_test.go: line 24 accesses `result.Result` which doesn't exist on `*ExtractionResult`
   - Root cause: alef e2e generator doesn't skip fixture categories when no fixture files exist; instead generates broken stub tests

## Bucket B — Fixture/Test Bugs

Issues in test fixtures (tools/benchmark-harness/fixtures/*.json) or alef.toml call overrides.

(To be updated as failures are discovered and triaged)

## Bucket C — Kreuzberg Core Bugs

Issues in kreuzberg core (crates/kreuzberg/src/) requiring code changes.

(To be updated as failures are discovered and triaged)

## Analysis

### Root Cause: Fixture-Test Mismatch

The Go e2e test suite cannot compile due to alef v0.14.3 codegen bugs. The issue stems from `alef.toml [crates.e2e]` pointing to benchmark-harness fixtures that:

1. Have categories like "image", "markup", "archive" (document classification), NOT "batch", "cache_operations", "contract", etc. (API/feature classification)
2. Don't include explicit "input.items" arrays for batch operations
3. Map to single-file extraction tests, not multi-file batch tests

Alef should either:

- Skip fixture categories with missing mappings, OR
- Provide explicit e2e fixtures in a separate directory with proper structure
- Or validate fixture structure before code generation

### Workaround Attempts

- Cannot edit `e2e/go/*_test.go` (auto-generated, explicit task constraint)
- Cannot rebuild without these broken test categories since alef generates them from fixture categories found
- Cannot create fixtures that would generate correct Go code (alef codegen patterns are defined in alef, not by fixture structure)

## Test Summary

- Total: Unable to determine (test suite doesn't compile)
- Passed: 0 (compilation failure)
- Failed: Compilation error in batch_test.go, cache_operations_test.go
- Skipped: All (due to compilation failure)

### Compilation Errors by Category

#### batch_test.go (6 failures)

- Line 20: Unmarshal target type is `[]string` instead of `[]BatchBytesItem`
- Lines 23-63: Multiple calls pass wrong types (nil strings, wrong array types)

#### cache_operations_test.go (4 failures)

- Lines 20, 29, 37, 45: Pass `nil` where string path expected
- Line 24: Accesses non-existent field `.Result` on ExtractionResult

#### Other test files

- Presumed working but untestable due to compilation failure
