package io.crates.moka.cache.simulator.policy.product;

import static com.github.benmanes.caffeine.cache.simulator.policy.Policy.Characteristic.WEIGHTED;

import java.util.Collections;
import java.util.HashSet;
import java.util.Set;

import com.github.benmanes.caffeine.cache.simulator.BasicSettings;
import com.github.benmanes.caffeine.cache.simulator.policy.AccessEvent;
import com.github.benmanes.caffeine.cache.simulator.policy.Policy;
import com.github.benmanes.caffeine.cache.simulator.policy.Policy.PolicySpec;
import com.github.benmanes.caffeine.cache.simulator.policy.PolicyStats;
import com.typesafe.config.Config;

@PolicySpec(name = "product.Moka", characteristics = WEIGHTED)
public final class MokaPolicy implements Policy {
  private final long cachePointer;
  private final PolicyStats policyStats;

  static {
    System.loadLibrary("moka");
  }

  public MokaPolicy(Config config, String evictionPolicy) {
    policyStats = new PolicyStats(name() + " (%s)", evictionPolicy);
    BasicSettings settings = new BasicSettings(config);
    long maximumSize = settings.maximumSize();
    boolean isWeighted = false;
    cachePointer = initCache(maximumSize, isWeighted, evictionPolicy);
  }

  // public MokaPolicy(Config config, Set<Characteristic> characteristics) {
  //   policyStats = new PolicyStats(name());
  //   BasicSettings settings = new BasicSettings(config);
  //   long maximumSize = settings.maximumSize();
  //   boolean isWeighted = characteristics.contains(WEIGHTED);
  //   String evictionPolicy = "TinyLFU";
  //   cachePointer = initCache(maximumSize, isWeighted, evictionPolicy);
  // }

  public static Set<Policy> policies(Config config) {
    HashSet<MokaPolicy> policies = new HashSet<>();
    policies.add(new MokaPolicy(config, "TinyLFU"));
    policies.add(new MokaPolicy(config, "LRU"));
    return Collections.unmodifiableSet(policies);
  }

  @Override
  public void record(AccessEvent event) {
    int value = getFromCacheIfPresent(cachePointer, event.key());
    if (value == -1) {
      putToCache(cachePointer, event.key(), event.weight());
      policyStats.recordWeightedMiss(event.weight());
    } else {
      policyStats.recordWeightedHit(event.weight());
      if (event.weight() != value) {
        putToCache(cachePointer, event.key(), event.weight());
      }
    }
  }

  @Override
  public void finished() {
    dropCache(cachePointer);
  }

  @Override
  public PolicyStats stats() {
    return policyStats;
  }

  /* ---------------------------------------------------------------------------
   * Native (Rust) functions to create and drive Moka cache.
   * --------------------------------------------------------------------------- */

  /**
   * Creates an instance of the Moka cache with given parameters and returns
   * the pointer to the instance as a <code>long</code> value.  Currently,
   * <code>moka::sync::Cache</code> is used.
   *
   * @param maximumSize
   * @param isWeighted
   * @param evictionPolicy The eviction policy to use. <code>TinyLFU</code> or
   *                      <code>LRU</code>.
   * @return The pointer to the instance of the Moka cache.
   */
  private static native long initCache(long maximumSize, boolean isWeighted, String evictionPolicy);

  /**
   * Returns the value (which is the weight in int) of the given key if
   * exists. Otherwise returns -1.
   *
   * @param cachePointer
   * @param key
   * @return The weight of the key if exists. Otherwise -1.
   */
  private static native int getFromCacheIfPresent(long cachePointer, long key);

  /**
   * Stores the value (which is the weight in int) for the given key.
   * Updates the value if already exists.
   *
   * @param cachePointer
   * @param key
   * @param weight
   */
  private static native void putToCache(long cachePointer, long key, int weight);

  /**
   * Drop the cache.
   *
   * @param cachePointer
   */
  private static native void dropCache(long cachePointer);

}
