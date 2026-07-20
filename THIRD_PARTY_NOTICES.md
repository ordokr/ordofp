# Third-Party Notices

This file documents third-party code used in OrdoFP and their licenses.

## Reference Repositories

OrdoFP uses patterns and algorithms from the following repositories. Where a
module was ported, its directory carries an `ORIGINAL_SOURCE.md` with the
detailed provenance.

> **Pin note:** the original commit-pin manifest was not preserved, so pins
> that cannot be verified are marked *not recorded* rather than guessed.
> Per-module `ORIGINAL_SOURCE.md` files remain authoritative where they exist.

### wgpu
- **Source**: https://github.com/gfx-rs/wgpu
- **License**: MIT OR Apache-2.0
- **Usage**: Device initialization patterns for GPU backend
- **Files**: `core/src/par/backend/wgpu/device.rs`, `core/src/par/backend/wgpu/buffer.rs` (see `core/src/par/backend/wgpu/ORIGINAL_SOURCE.md`)
- **Pinned Commit**: `0ac2da4b6baea94c1e9d9bb03bdcf69122da857a`

### burn
- **Source**: https://github.com/tracel-ai/burn
- **License**: MIT OR Apache-2.0
- **Usage**: WGSL codegen architecture patterns
- **Files**: `core/src/par/codegen/wgsl.rs`
- **Pinned Commit**: not recorded (manifest lost; see pin note above)

### frunk
- **Source**: https://github.com/lloydmeta/frunk
- **License**: MIT
- **Usage**: OrdoFP's HList, labelled-generic, coproduct, path, tuple-conversion,
  and derive/proc-macro architecture is derived from frunk, with types renamed to
  Latin (`HNil`→`Nihil`, `HCons`→`Coniunctio`, `Generic`→`Universalis`,
  `LabelledGeneric`→`NominataUniversalis`, `Coproduct`→`Disiunctio`,
  `Validated`→`Probatum`, `Transmogrifier`→`Transfigurator`) and substantially
  extended/rewritten since.
- **Files**: `core/src/{hlist,labelled,indices,path,tuples,traits,macros,universalis,disiunctio}.rs`,
  `src/{validated,semigroup,monoid}.rs`, `derives/`, `proc_macros/`, `proc_macro_helpers/`
- **Pinned Commit**: not recorded (derivation predates the pin manifest; see pin note above)

Copyright notice preserved per the MIT license:

> Copyright (c) 2016 Lloyd Chan
>
> Permission is hereby granted, free of charge, to any person obtaining a copy of
> this software and associated documentation files (the "Software"), to deal in
> the Software without restriction, including without limitation the rights to
> use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
> the Software, and to permit persons to whom the Software is furnished to do so,
> subject to the following conditions:
>
> The above copyright notice and this permission notice shall be included in all
> copies or substantial portions of the Software.

### futhark
- **Source**: https://github.com/diku-dk/futhark
- **License**: ISC
- **Usage**: GPU memory coalescing and buffer reuse patterns
- **Files**: `core/src/par/opt/buffer_pool.rs` (earlier layout and transpose-kernel adaptations have been removed from the tree)
- **Pinned Commit**: `ef8779cee18bdfebd9af05b41c7f3f1631933181`

Copyright notice preserved per the ISC license:

> Copyright (c) 2013-2022. DIKU, University of Copenhagen

### purescript-free
- **Source**: https://github.com/purescript/purescript-free
- **License**: BSD-3-Clause
- **Usage**: Church-encoded Free monad patterns for CPS transformers
- **Files**: `core/src/transformers/ecclesia/` (LectorEcclesiaT; four further variants adapted from the same source were removed from the tree)
- **Pinned Commit**: not recorded (manifest lost; see pin note above)

Copyright notice preserved per the BSD-3-Clause license:

> Copyright 2018 PureScript

### monad-bayes
- **Source**: https://github.com/tweag/monad-bayes
- **License**: MIT
- **Usage**: Bayesian inference algorithms
- **Files**: `ordofp_bayes/src/` (see `ordofp_bayes/ORIGINAL_SOURCE.md`)
- **Pinned Commit**: not recorded here — see `ordofp_bayes/ORIGINAL_SOURCE.md`

Copyright notice preserved per the MIT license:

> Copyright (c) 2015-2020 Adam Scibior

### rayon
- **Source**: https://github.com/rayon-rs/rayon
- **License**: MIT OR Apache-2.0
- **Usage**: Parallel iterator patterns for ParFlumen CPU backend
- **Files**: `core/src/par/backend/mod.rs` (CpuRayon implementation)
- **Pinned Commit**: not recorded (manifest lost; see pin note above)

### vector (Haskell)
- **Source**: https://github.com/haskell/vector
- **License**: BSD-3-Clause
- **Usage**: Stream fusion patterns (Step/Stream model, from `src/Data/Stream/Monadic.hs` of vector-stream)
- **Files**: `core/src/async_core/flumen_fusus.rs`
- **Pinned Commit**: `8943433f6f8432235db9826f206bcba329e6bde5`

Copyright notice preserved per the BSD-3-Clause license:

> Copyright (c) 2008-2012, Roman Leshchinskiy;
> 2020-2022, Alexey Kuleshevich; 2020-2022, Aleksey Khudyakov;
> 2020-2022, Andrew Lelechenko

## Design References (patterns only, no code copied)

API and architecture patterns were also studied in the following projects;
no code was copied from them:

- **rustica** (https://crates.io/crates/rustica) — typeclass/datatype API shapes (`Identity`, `Writer`, `ContT`, contravariant/profunctor traits)
- **rust2fun** (https://crates.io/crates/rust2fun) — contravariant functor API shape
- **transfigure** (https://github.com/ivan-m/transfigure) — the field-reordering record-conversion idea behind `Transfigurator`
- **rsmpeg** (https://github.com/larksuite/rsmpeg) — safe C-wrapping architecture patterns (`ffi_bedrock`)
- **rust-fp** (https://github.com/j5ik2o/rust-fp) — persistent-data-structure API shapes (`pfds`)
- **wide** (https://crates.io/crates/wide) — SIMD wrapper-type API shapes (`par::simd`)

## License Compatibility & Attribution

All code in OrdoFP core is licensed under **Apache-2.0**. Patterns and algorithms from the repositories above are adapted (not copied verbatim) and are compatible with Apache-2.0 (MIT/BSD/ISC notices preserved where required).
