package io.crates.moka.cache.simulator.policy.product;

import static com.github.benmanes.caffeine.cache.simulator.policy.Policy.Characteristic.WEIGHTED;

import java.util.Set;

import com.github.benmanes.caffeine.cache.simulator.BasicSettings;
import com.github.benmanes.caffeine.cache.simulator.policy.AccessEvent;
import com.github.benmanes.caffeine.cache.simulator.policy.Policy;
import com.github.benmanes.caffeine.cache.simulator.policy.Policy.PolicySpec;
import com.github.benmanes.caffeine.cache.simulator.policy.PolicyStats;
import com.typesafe.config.Config;

@PolicySpec(name = "product.Moka", characteristics = WEIGHTED)
public final class MokaPolicy implements Policy {
  private final PolicyStats policyStats;

  static {
    System.loadLibrary("moka");
  }

  public MokaPolicy(Config config, Set<Characteristic> characteristics) {
    policyStats = new PolicyStats(name());
    BasicSettings settings = new BasicSettings(config);
    long maximumSize = settings.maximumSize();
    boolean isWeighted = characteristics.contains(WEIGHTED);
    initCache(maximumSize, isWeighted);
  }

  @Override
  public void record(AccessEvent event) {
    int value = getFromCacheIfPresent(event.key());
    if (value == -1) {
      putToCache(event.key(), event.weight());
      policyStats.recordWeightedMiss(event.weight());
    } else {
      policyStats.recordWeightedHit(event.weight());
      if (event.weight() != value) {
        putToCache(event.key(), event.weight());
      }
    }
  }

  // @Override
  // public void finished() {
  //
  // }

  @Override
  public PolicyStats stats() {
    return policyStats;
  }

  /* ---------------------------------------------------------------------------
   * Native (Rust) functions to create and drive Moka cache.
   * --------------------------------------------------------------------------- */

  /**
   * Creates the shared singleton instance of the Moka cache with given
   * parameters. Currently, moka::sync::Cache is used.
   *
   * @param maximumSize
   * @param isWeighted
   */
  private static native void initCache(long maximumSize, boolean isWeighted);

  /**
   * Returns the value (which is the weight in int) of the given key if
   * exists. Otherwise returns -1.
   *
   * @param key
   * @return The weight of the key if exists. Otherwise -1.
   */
  private static native int getFromCacheIfPresent(long key);

  /**
   * Stores the value (which is the weight in int) for the given key.
   * Updates the value if already exists.
   *
   * @param key
   * @param weight
   */
  private static native void putToCache(long key, int weight);

}
