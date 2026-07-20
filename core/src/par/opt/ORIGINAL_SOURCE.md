# Original Source Attribution

## Futhark Layout Transforms and Buffer Reuse

**Source Repository**: https://github.com/diku-dk/futhark  
**License**: ISC  
**Pinned Commit**: `ef8779cee18bdfebd9af05b41c7f3f1631933181`

### Adapted Files
- `core/src/par/opt/buffer_pool.rs` - Interference graph and buffer reuse

(Earlier adaptations — `layout.rs` and a tile-based transpose kernel — have
been removed from the tree; this attribution is retained for the surviving
file only.)

### Changes
- Adapted to Rust syntax
- Integrated with OrdoFP ParFlumen architecture
- Simplified for MVP implementation
