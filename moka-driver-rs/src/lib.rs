use jni::{
    objects::{JClass, JString},
    sys::{jboolean, jint, jlong},
    JNIEnv,
};
use moka::{policy::EvictionPolicy, sync::Cache};
use std::hash::BuildHasher;

const HASH_SEED_KEY: u64 = 982922761776577566;

type CacheTy = Cache<i64, i32, DefaultHasher>;

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

#[no_mangle]
pub extern "system" fn Java_io_crates_moka_cache_simulator_policy_product_MokaPolicy_initCache<
    'local,
>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    maximum_capacity: jlong,
    is_weighted: jboolean,
    eviction_policy: JString<'local>,
) -> jlong {
    let mut builder = Cache::builder().max_capacity(maximum_capacity as u64);

    if is_weighted == 0 {
        builder = builder.initial_capacity(maximum_capacity as usize);
    } else {
        builder = builder.weigher(|_k, v| *v as u32);
    }

    let ep: String = env
        .get_string(&eviction_policy)
        .expect("Could not get a Java String")
        .into();
    let eviction_policy = match ep.to_ascii_lowercase().as_str() {
        "tinylfu" => EvictionPolicy::TinyLfu,
        "lru" => EvictionPolicy::Lru,
        _ => panic!("Unknown eviction policy: {}", ep),
    };
    builder = builder.eviction_policy(eviction_policy);

    let cache: CacheTy = builder.build_with_hasher(DefaultHasher);
    Box::into_raw(Box::new(cache)) as jlong
}

#[no_mangle]
pub extern "system" fn Java_io_crates_moka_cache_simulator_policy_product_MokaPolicy_getFromCacheIfPresent<
    'local,
>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    cache_ptr: jlong,
    key: jlong,
) -> jint {
    let cache = unsafe { &mut *(cache_ptr as *mut CacheTy) };
    cache.get(&key).unwrap_or(-1)
}

#[no_mangle]
pub extern "system" fn Java_io_crates_moka_cache_simulator_policy_product_MokaPolicy_putToCache<
    'local,
>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    cache_ptr: jlong,
    key: jlong,
    value: jint,
) {
    let cache = unsafe { &mut *(cache_ptr as *mut CacheTy) };
    cache.insert(key, value);
}

#[no_mangle]
pub extern "system" fn Java_io_crates_moka_cache_simulator_policy_product_MokaPolicy_dropCache<
    'local,
>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    cache_ptr: jlong,
) {
    let _boxed_cache = unsafe { Box::from_raw(cache_ptr as *mut CacheTy) };
}
