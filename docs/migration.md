# Lineage & Version Notes

> **Lineage note:** OrdoFP began as a fork of [frunk](https://github.com/lloydmeta/frunk)
> (MIT). The core HList/labelled/coproduct machinery is a renamed and extended
> derivative — see `THIRD_PARTY_NOTICES.md` for the attribution and the rename map
> in `docs/glossary.md`.

> *"Transitus sine periculo."*
> — Safe passage.

## The version reset (2.x → 0.1.0)

The project reset its versioning from an internal 2.x line to 0.1.0
(2026-04-25) to reflect the crate's then-unpublished, pre-1.0 status. 0.1.0 is
therefore the release *after* the internal 2.0, despite the smaller number.
0.1.0 is the first published line; if you encounter references to 2.x
version labels (e.g. in `docs/glossary.md` history notes), they predate the
reset.

## Notes for readers of older internal code

- The v1 row API (`effects::row::*` — `RowExtensio`, `RowVacuus`,
  `EffectusRow`, `HasEffectus`) was consolidated into `effects::row_v2`
  (const-generic `EffectSet<MASK>`); import from `row_v2` directly.
- The runtime behind the `async-std` feature is now **smol** (async-std is
  discontinued); `async-std` remains only as a back-compat alias.
- Two distinct resource-bracket types exist: the sync `ordofp_core::linear::Res`
  (feature `linear`) and the async `ordofp_core::async_core::res::Res`
  (feature `async`). Same name, different types and APIs.
- `dependent`, `quantitative`, `rows`, `distributed`, `supervision`, and
  `ffi` are off-by-default; enable the matching feature to restore access.

For the canonical feature matrix see `docs/FEATURE_FLAGS.md`; for the API
reference see `docs/reference.md`; for terminology see `docs/glossary.md`.
