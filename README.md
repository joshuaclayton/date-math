# date-math

![CI](https://github.com/joshuaclayton/date-math/workflows/CI/badge.svg)
![Security audit](https://github.com/joshuaclayton/date-math/workflows/Security%20audit/badge.svg)

A small CLI for doing date calculations.

## Usage / Examples

### Full date plus a duration

```sh
date-math 'dec 30, 2021 + 2 weeks + 1 day'
2022-01-14
```

### Arbitrary durations

Given a date of July 2, 2021:

```sh
date-math '2 weeks + 3 days'
2021-07-19
```

## Installation

Given a working installation of Rust:

```sh
git clone git@github.com:joshuaclayton/date-math.git
cd date-math
cargo install --path .
```

## License

Copyright 2021 Josh Clayton. See the [LICENSE](LICENSE).
