# Original Source Attribution

## PureScript Free Monad (Church Encoding)

**Source Repository**: https://github.com/purescript/purescript-free  
**License**: BSD-3-Clause  
**Pinned Commit**: `4329227581c08bb2a3dd6315f7d447f56d99088f`  
**Original File**: `src/Control/Monad/Free.purs`

### Adapted Files
- `core/src/transformers/ecclesia/lector_ecclesia.rs` - CPS ReaderT

(Four further CPS variants — StateT, OptionT, EitherT, WriterT — were adapted
from the same source and later removed from the tree.)

### Changes
- Adapted from PureScript to Rust
- Integrated with OrdoFP Monad trait
- Added O(1) bind composition via CPS
