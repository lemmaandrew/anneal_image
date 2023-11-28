# anneal_image
Tool that uses simulated annealing to recreate images

Usage: `cargo run -- input\_image.extension output\_image.extension [alpha]`

`alpha` is an optional argument (defaults to 0.999) which determines the rate at which the
program's "temperature" changes. Values close to 1 will cause the temperature to decrease slowly,
while values closer to 0 will cause the temperature to decrease rapidly.

The program finishes annealing when the temperature, which starts at 1000 and is printed to STDOUT, reaches 0.001.
