# Original Source Attribution

## monad-bayes (Haskell)

**Source Repository**: https://github.com/tweag/monad-bayes  
**License**: MIT

### Adapted Files
- `ordofp_bayes/src/traits.rs` - Samplandus/Inferendus traits (from `Control.Monad.Bayes.Class`, MonadSample/MonadInfer)
- `ordofp_bayes/src/inference.rs` - SMC, Metropolis-Hastings, Importance Sampling algorithms
  (from `Control.Monad.Bayes.Traced.Common`, `Control.Monad.Bayes.Population`,
  `Control.Monad.Bayes.Inference.SMC`)
- `ordofp_bayes/src/distributions.rs` - distribution patterns

### Changes
- Adapted from Haskell to Rust
- Integrated with OrdoFP effect system
- Added ParFlumen/Rayon for parallel particle updates
