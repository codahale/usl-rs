# usl

`usl` is a Rust modeler for [Dr. Neil Gunther][NJG]'s [Universal Scalability Law][USL] as described
in [Baron Schwartz][BS]'s book [Practical Scalability Analysis with the Universal Scalability Law][PSA].

Given a handful of measurements of any two [Little's Law][LL] parameters--throughput, latency, and
concurrency--the [USL][USL] allows you to make predictions about any of those parameters' values given an arbitrary
value for any another parameter. For example, given a set of measurements of concurrency and throughput, the [USL][USL]
will allow you to predict what a system's average latency will look like at a particular throughput, or how many servers
you'll need to process requests and stay under your SLA's latency requirements.

The model coefficients and predictions should be within 0.001% of those listed in the book.

## How to use this

As an example, consider doing load testing and capacity planning for an HTTP server. To model the behavior of the system
using the [USL][USL], you must first gather a set of measurements of the system. These measurements must be of two of
the three parameters of [Little's Law][LL]: mean response time (in seconds), throughput (in requests per second), and
concurrency (i.e. the number of concurrent clients).

Because response time tends to be a property of load (i.e. it rises as throughput or concurrency rises), the dependent
variable in your tests should be mean response time. This leaves either throughput or concurrency as your independent
variable, but thanks to [Little's Law][LL] it doesn't matter which one you use. For the purposes of discussion, let's
say you measure throughput as a function of the number of concurrent clients working at a fixed rate (e.g. you used
[`wrk2`][wrk2]).

After you're done load testing, you should have a set of measurements shaped like this:

|concurrency|throughput|
|-----------|----------|
|          1|        65|
|         18|       996|
|         36|      1652|
|         72|      1853|
|        108|      1829|
|        144|      1775|
|        216|      1702|

Now you can build a model and begin estimating things.

### As A CLI Tool

```
cargo install usl --features=cli
```

```
$ cat measurements.csv
1,65
18,996
36,1652
72,1853
etc.
```

```
usl --plot example.csv 10 50 100 150 200 250 300

USL parameters: σ=0.028168, κ=90.691376, λ=0.000104
	max throughput: 1882.421555, max concurrency: 96.000000
	contention constrained
⡁⠈ ⡡⠊⠙⠑⢚⢄⡁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⡁ 1882.2
⠄ ⡜      ⠈⠑⠴⡀                                                                                       ⠄
⠂⢰⠁         ⠈⠑⠢⣀                                                                                    ⠂
⡁⡜              ⠉⠒⠤⣀                                                                                ⡁
⠄⡇                  ⠉⠒⠤⢄⡀                                                                           ⠄
⢢⠃                      ⠈⠉⠒⠢⠤⣀⡀                                                                     ⠂
⣹⠁                            ⠈⠉⠑⠒⠢⠤⢄⣀⡀                                                             ⡁
⢼                                     ⠈⠉⠉⠒⠒⠢⠤⠤⢄⣀⣀⡀                                                  ⠄
⠊                                                ⠈⠉⠉⠉⠒⠒⠒⠒⠤⠤⠤⠤⢄⣀⣀⣀⣀⡀                                 ⠂
⡁                                                                 ⠈⠉⠉⠉⠉⠉⠒⠒⠒⠒⠒⠒⠢⠤⠤⠤⠤⠤⠤⠤⣀⣀⣀⣀⣀⣀⣀⣀⣀     ⡁
⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈⠉⠉⠉⠉⠉  405.7
0.0                                                                                              1882.4
10,718.1341832148264
50,1720.7701516725795
100,1881.977293350178
150,1808.2668068616638
200,1687.6636402563477
250,1564.4594617061496
300,1450.4659509826192
```

### As A Library

```rust
use usl::{Model, Measurement};

fn main() {
    let measurements = vec![
        Measurement::concurrency_and_throughput(1, 65.0),
        Measurement::concurrency_and_throughput(18, 996.0),
        Measurement::concurrency_and_throughput(36, 1652.0),
        Measurement::concurrency_and_throughput(72, 1853.0),
        Measurement::concurrency_and_throughput(108, 1829.0),
        Measurement::concurrency_and_throughput(144, 1775.0),
        Measurement::concurrency_and_throughput(216, 1702.0),
    ];
    let model = Model::build(&measurements);
    println!("{}", model.throughput_at_concurrency(100));
}
```

## Performance

Building models is pretty fast:

```
build                   time:   [9.4531 us 9.4605 us 9.4677 us]                   
```

## Further reading

I strongly recommend [Practical Scalability Analysis with the Universal Scalability Law][PSA], a free e-book
by [Baron Schwartz][BS], author of [High Performance MySQL][MySQL] and CEO of [VividCortex][VC]. Trying to use this
library without actually understanding the concepts behind [Little's Law][LL], [Amdahl's Law][AL], and
the [Universal Scalability Law][USL] will be difficult and potentially misleading.

I also [wrote a blog post about my Java implementation of USL][usl4j].

## License

Copyright © 2021 Coda Hale

Distributed under the Apache License 2.0 or MIT License.

[NJG]: http://www.perfdynamics.com/Bio/njg.html

[AL]: https://en.wikipedia.org/wiki/Amdahl%27s_law

[LL]: https://en.wikipedia.org/wiki/Little%27s_law

[PSA]: https://www.vividcortex.com/resources/universal-scalability-law/

[USL]: http://www.perfdynamics.com/Manifesto/USLscalability.html

[BS]: https://www.xaprb.com/

[MySQL]: http://shop.oreilly.com/product/0636920022343.do

[VC]: https://www.vividcortex.com/

[wrk2]: https://github.com/giltene/wrk2

[usl4j]: https://codahale.com/usl4j-and-you/
