mod cache;
pub mod cache_cx;
mod call_id;
mod gap_buffer;

pub use cache::{Cache, CacheVar};
pub use call_id::CallId;
pub use kyute_compose_macros::{composable, Widget};

#[cfg(test)]
mod tests {
    use crate::{cache::Cache, cache_cx as ccx, CacheVar};
    use std::sync::Arc;

    //==================================================
    fn composable() -> Arc<CacheVar<u32>> {
        ccx::enter_call(0);
        let (var, _) = ccx::variable(|| 42u32);
        ccx::exit_call();
        var
    }

    fn composable2(var: Arc<CacheVar<u32>>) -> u32 {
        ccx::memoize((), move || {
            // introduce a dependency on the value of the variable by reading from it
            var.set_dependency();
            // bikeshed: var.track(), var.track_here()
            let value = var.get();
            value + 1
        })
    }

    fn main_composable() -> (Arc<CacheVar<u32>>, u32) {
        ccx::enter_call(0);
        let var = composable();
        let varplusone = composable2(var.clone());
        ccx::exit_call();
        (var, varplusone)
    }

    //==================================================
    #[test]
    fn it_works() {
        let waker = dummy_waker::dummy_waker();
        let mut cache = Cache::new(waker);

        let (var, varp) = cache.run(|| main_composable());
        assert_eq!(var.get(), 42);
        assert_eq!(varp, 43);
        var.replace(63, true);
        assert_eq!(var.get(), 63);
        cache.dump();
        let (var, varp) = cache.run(|| main_composable());
        assert_eq!(var.get(), 63);
        assert_eq!(varp, 64);
        cache.dump();
        let (var, varp) = cache.run(|| main_composable());
        assert_eq!(var.get(), 63);
        assert_eq!(varp, 64);
        cache.dump();
    }
}
