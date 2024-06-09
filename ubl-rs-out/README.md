# ubl-rs

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)

For working with [UBL](https://docs.oasis-open.org/ubl/os-UBL-2.1/UBL-2.1.html) in rust.

## Description

Generated crate (library) for working with Universal Business Language from within a rust project.

Project that generated this library is open sourced as well, see [ubl-rs-builder](https://github.com/hienohomma/ubl-rs-builder)

No modifications should be made outside of [exporter.rs](src/exporter.rs) file. Every execution of [ubl-rs-builder](https://github.com/hienohomma/ubl-rs-builder) overwrites the [src/](src) directory with the exception of [exporter.rs](src/exporter.rs).

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)

## Installation

Include this library in your projects' `Cargo.toml` file and off you go.

``` toml
ubl-rs = { git = "https://github.com/hienohomma/ubl-rs.git" }
```

## Usage

See example of a trivial invoice creation from [ubl-rs-tester](https://github.com/hienohomma/ubl-rs-tester)  
Other examples of UBL json content can be found from [oasis-open.org](http://docs.oasis-open.org/ubl/UBL-2.1-JSON/v1.0/cnd02/json/)

## Contributing

Pull requests, reported issues, improvements in documentation etc. are always welcome.  
Try to behave while at it.

__All improvements should be made to the [ubl-rs-builder](https://github.com/hienohomma/ubl-rs-builder) project__

## License

This project is licensed under the [MIT License](https://opensource.org/licenses/MIT).

## Contact

- Email: <opensource@hienohomma.fi>
- GitHub: [hienohomma](https://github.com/hienohomma)
