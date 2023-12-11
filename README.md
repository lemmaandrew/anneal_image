# anneal_image
Tool that uses simulated annealing to recreate images

Usage: `cargo run -- --input input-image.extension --output output-image.extension [--alpha alpha] [--triangle] [--sample sample] [--multithreading]`

`alpha` is an optional argument (defaults to 0.999) which determines the rate at which the
program's "temperature" changes. Values close to 1 will cause the temperature to decrease slowly,
while values closer to 0 will cause the temperature to decrease rapidly.

`triangle` is an optional flag which switches the drawn shapes from rectangles to triangles.
In my personal opinion, this looks better at high alphas than rectangles at the same alphas.

`sample` is an optional argument which turns the cost function into a sampling cost function.
Don't worry about it, it makes the program run faster at the trade-off of accuracy.

`multithreading` is an optional flag which enables some multithreading capabilities. At the moment, this unilaterally makes
the program slower, but I'm working on it don't worry.

The program finishes annealing when the temperature, which starts at 1000 and is printed to STDOUT, reaches 0.001.
