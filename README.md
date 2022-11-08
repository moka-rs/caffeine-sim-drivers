# Moka Cache Driver for the Caffeine Simulator

This repository contains a [Moka cache][moka-cache] driver for the
[Caffeine Simulator][caffeine-simulator]. The driver enables the Caffeine Simulator
to run workloads against a Moka cache to generate [charts][moka-perf-charts] with
hit vs. miss ratios.

[moka-cache]: https://github.com/moka-rs/moka
[caffeine-simulator]: https://github.com/ben-manes/caffeine/wiki/Simulator
[moka-perf-charts]: https://github.com/moka-rs/moka/wiki#benchmarks-hit-ratio

## How does it work?

The Caffeine Simulator is written in Java but Moka cache is written in Rust. This
driver uses Java Native Interface ([JNI][jni]) to bridge the gap between the two
languages.

The driver consists of two parts:

1. A Java class that implements the `Policy` interface of the Simulator. It calls
   `native` (Rust) functions.
2. A Rust library that wraps Moka cache and implements the functions called by the
   Java class.
    - This library uses [jni crate][jni-crate], which provides a safe wrapper around
      the JNI API.

The Rust library is compiled into a dynamic library that is loaded into the Java VM
at runtime.

[jni]: https://en.wikipedia.org/wiki/Java_Native_Interface
[jni-crate]: https://crates.io/crates/jni

## Prerequisites

- Java JDK to build the Caffeine Simulator and the Java part of the driver.
- Rust stable toolchain (1.51 or newer) to build Moka and the Rust part of the driver.

## Building the Driver

Suppose you use `~/sim` as the working directory.

Clone this repository:

```console
$ SIM=~/sim
$ cd $SIM
$ git clone https://github.com/moka-rs/caffeine-sim-drivers.git
```

Build the Rust part of the driver:

```console
$ cd $SIM/caffeine-sim-drivers/moka-driver-rs
$ cargo build --release
$ DRV_LIB=$SIM/caffeine-sim-drivers/moka-driver-rs/target/release
```

Clone Caffeine's repository, and checkout a specific Git revision:

```console
$ REVISION=6800aa6573361e440c77d58b22e54c16d0ce2505

$ cd $SIM
$ git clone https://github.com/ben-manes/caffeine.git
$ (cd caffeine && git checkout $REVISION)
```

Copy the Java part of the driver into the Caffeine repository:

```console
$ POL_DIR=simulator/src/main/java/io/crates/moka/cache/simulator/policy/product/
$ mkdir -p $SIM/caffeine/$POL_DIR
$ cp -p $SIM/caffeine-sim-drivers/moka-driver-java/MokaPolicy.java $SIM/caffeine/$POL_DIR/
```

Copy a patch file into the Caffeine repository:

```console
$ cp -p $SIM/caffeine-sim-drivers/moka-driver-java/registry-patch.diff $SIM/caffeine/
```

Apply the patch:

```console
$ cd $SIM/caffeine
$ git apply registry-patch.diff
```

## Running the Simulator

Create `application.conf` from the template:

```console
$ cd $SIM/caffeine/simulator/src/main/resources/
$ cp -p reference.conf application.conf
```

Edit `application.conf` and add the following line in the `policies` section:

```properties
  policies = [
    # ...
    product.Moka,
  ]
```

Build and run the Caffeine Simulator:

```console
## Replace `/path/to/trace/S3.lis` with the real path to the trace file.

$ ./gradlew simulator:simulate -q \
    -Dcaffeine.simulator.files.paths.0=arc:/path/to/trace/S3.lis \
    --maximumSize=100_000,200_000,300_000,400_000,500_000,600_000,700_000,800_000 \
    --jvmArgs="-XX:+UseParallelGC,-Xmx8g,-Djava.library.path=$DRV_LIB" \
    --theme=light
```

## Modifying the Driver

If you want to modify the driver, e.g., to drive your own cache implementation, check
out the driver's codes and the "Getting Started" section of the jni crate's
documentation:

- Driver's source code:
    - Java part: [MokaPolicy.java](./moka-driver-java/MokaPolicy.java)
    - Rust part: [src/lib.rs](./moka-driver-rs/src/lib.rs)
- jni crate: [Getting Started][jni-crate-getting-started]

[jni-crate-getting-started]: https://docs.rs/jni/latest/jni/index.html#getting-started

## License

The Apache License 2.0. See [LICENSE](./LICENSE) for details.

