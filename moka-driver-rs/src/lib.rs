use jni::{
    objects::JClass,
    sys::{jboolean, jint, jlong},
    JNIEnv,
};
use moka::sync::Cache;
use once_cell::sync::OnceCell;
use std::hash::BuildHasher;

const HASH_SEED_KEY: u64 = 982922761776577566;

#[derive(Clone, Default)]
pub(crate) struct DefaultHasher;

impl BuildHasher for DefaultHasher {
    // Picking a fast but also good algorithm by default to avoids weird scenarios in
    // some implementations (e.g. poor hashbrown performance, poor bloom filter
    // accuracy). Algorithms like FNV have poor quality in the low bits when hashing
    // small keys.
    type Hasher = xxhash_rust::xxh3::Xxh3;

    fn build_hasher(&self) -> Self::Hasher {
        xxhash_rust::xxh3::Xxh3Builder::new()
            .with_seed(HASH_SEED_KEY)
            .build()
    }
}

static CACHE: OnceCell<Cache<i64, i32, DefaultHasher>> = OnceCell::new();

fn shared_cache() -> &'static Cache<i64, i32, DefaultHasher> {
    CACHE.get().expect("The cache is not initialized")
}

#[no_mangle]
pub extern "system" fn Java_io_crates_moka_cache_simulator_policy_product_MokaPolicy_initCache(
    _env: JNIEnv,
    _class: JClass,
    maximum_capacity: jlong,
    is_weighted: jboolean,
) {
    let mut builder = Cache::builder().max_capacity(maximum_capacity as u64);
    if is_weighted == 0 {
        builder = builder.initial_capacity(maximum_capacity as usize);
    } else {
        builder = builder.weigher(|_k, v| *v as u32);
    }
    let cache = builder.build_with_hasher(DefaultHasher::default());
    let _ = CACHE.set(cache);
}

#[no_mangle]
pub extern "system" fn Java_io_crates_moka_cache_simulator_policy_product_MokaPolicy_getFromCacheIfPresent(
    _env: JNIEnv,
    _class: JClass,
    key: jlong,
) -> jint {
    if let Some(v) = shared_cache().get(&key) {
        v
    } else {
        -1
    }
}

#[no_mangle]
pub extern "system" fn Java_io_crates_moka_cache_simulator_policy_product_MokaPolicy_putToCache(
    _env: JNIEnv,
    _class: JClass,
    key: jlong,
    value: jint,
) {
    shared_cache().insert(key as i64, value as i32);
}
