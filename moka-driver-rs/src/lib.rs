use jni::{
    objects::{JClass, JString},
    sys::{jboolean, jint, jlong},
    JNIEnv,
};
use moka::{future::Cache as AsyncCache, policy::EvictionPolicy, sync::Cache as SyncCache};
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
    let cache_type = "async";

    let is_weighted = is_weighted != 0;

    let ep: String = env
        .get_string(&eviction_policy)
        .expect("Could not get a Java String")
        .into();
    let eviction_policy = match ep.to_ascii_lowercase().as_str() {
        "windowtinylfu" => EvictionPolicy::window_tiny_lfu().window_allocation(0.01),
        "tinylfu" => EvictionPolicy::tiny_lfu(),
        "lru" => EvictionPolicy::lru(),
        _ => panic!("Unknown eviction policy: {}", ep),
    };

    let cache = Cache::new(
        cache_type,
        maximum_capacity as u64,
        is_weighted,
        eviction_policy,
    );

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
    let cache = unsafe { &mut *(cache_ptr as *mut Cache) };
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
    let cache = unsafe { &mut *(cache_ptr as *mut Cache) };
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
    let _boxed_cache = unsafe { Box::from_raw(cache_ptr as *mut Cache) };
}

enum Cache {
    Async(AsyncCache<i64, i32, DefaultHasher>),
    Sync(SyncCache<i64, i32, DefaultHasher>),
}

impl Cache {
    fn new(
        cache_type: &str,
        maximum_capacity: u64,
        is_weighted: bool,
        eviction_policy: EvictionPolicy,
    ) -> Self {
        match cache_type {
            "async" => Self::build_async_cache(maximum_capacity, is_weighted, eviction_policy),
            "sync" => Self::build_sync_cache(maximum_capacity, is_weighted, eviction_policy),
            _ => panic!("Unknown cache type: {}", cache_type),
        }
    }

    fn get(&mut self, key: &i64) -> Option<i32> {
        match self {
            Cache::Async(cache) => smol::block_on(cache.get(key)),
            Cache::Sync(cache) => cache.get(key),
        }
    }

    fn insert(&mut self, key: i64, value: i32) {
        match self {
            Cache::Async(cache) => {
                smol::block_on(cache.insert(key, value));
            }
            Cache::Sync(cache) => {
                cache.insert(key, value);
            }
        }
    }

    fn build_async_cache(
        max_capacity: u64,
        is_weighted: bool,
        eviction_policy: EvictionPolicy,
    ) -> Self {
        let mut builder = AsyncCache::builder()
            .max_capacity(max_capacity)
            .eviction_policy(eviction_policy);

        if !is_weighted {
            builder = builder.initial_capacity(max_capacity as usize);
        } else {
            builder = builder.weigher(|_k, v| *v as u32);
        }

        Self::Async(builder.build_with_hasher(DefaultHasher))
    }

    fn build_sync_cache(
        max_capacity: u64,
        is_weighted: bool,
        eviction_policy: EvictionPolicy,
    ) -> Self {
        let mut builder = SyncCache::builder()
            .max_capacity(max_capacity)
            .eviction_policy(eviction_policy);

        if !is_weighted {
            builder = builder.initial_capacity(max_capacity as usize);
        } else {
            builder = builder.weigher(|_k, v| *v as u32);
        }

        Self::Sync(builder.build_with_hasher(DefaultHasher))
    }
}
