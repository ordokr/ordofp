#[cfg(all(feature = "par", feature = "gpu-wgpu"))]
mod tests {
    use ordofp_core::par::{GpuMapChain, Nodus, backend::wgpu::GpuWgpu};

    struct EvilNode;

    impl Nodus for EvilNode {
        type Item = String; // Not POD!

        fn len(&self) -> usize {
            1
        }

        fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
            sink("safe fallback".to_string());
        }

        // Return bytes for a float (e.g., 0.0)
        // If interpreted as String, this is a null pointer and 0 length/capacity?
        // Or garbage if we return something else.
        fn try_as_gpu_source(&self) -> Option<(&[u8], &'static str)> {
            // Return 24 bytes of garbage.
            static BYTES: [u8; 24] = [1; 24]; // Non-zero garbage
            Some((&BYTES, "f32"))
        }

        fn try_gpu_map_chain(&self) -> Option<GpuMapChain> {
            Some(GpuMapChain::new(1))
        }
    }

    #[test]
    fn test_collect_ub() {
        // Only run if we can create a GPU device
        let Ok(backend) = GpuWgpu::new() else {
            println!("Skipping test: No GPU available");
            return;
        };

        let node = EvilNode;

        // This should crash or produce UB if not protected
        use ordofp_core::par::backend::Backend;
        let result: Vec<String> = backend.collect(&node);
        assert_eq!(result, vec!["safe fallback".to_string()]);
    }
}
