# ubl-rs-builder

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)

Builds [ubl-rs](https://github.com/hienohomma/ubl-rs) crate from [UBL-2.1-JSON specification](https://docs.oasis-open.org/ubl/UBL-2.1-JSON/v1.0/UBL-2.1-JSON-v1.0.html)

## Description

Universal Business Language has [json schema](https://json-schema.org/) of its open format for sending and receieving invoices, quotes etc. through any means agreed between both parties.

This project builds [ubl-rs](https://github.com/hienohomma/ubl-rs) library which has the required structs, enums and type aliases for working with UBL.

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [License](#license)
- [Contributing](#contributing)
- [Contact](#contact)

## Installation

You need to [install rust](https://www.rust-lang.org/tools/install) and cargo to compile the binary.  
Only tested on linux, if binary compiles / program works on windows or mac that's completely unintentional.

## Usage

Clone this project, navigate to ubl-rs-builder dir and execute the binary.

`ubl-rs` directory containing the library will be created next to the `ubl-rs-builder` dir upon successful execution.

``` bash
cd ubl-rs-builder
cargo run
cd ../ubl-rs
```

To use `ubl-rs` lib in your local rust project, add it as a local dependency in `Cargo.toml`:

``` toml
ubl-rs = { path = "/path_to/ubl-rs" }
```

See [ubl-rs-tester](https://github.com/hienohomma/ubl-rs-tester) for an example how to recreate [invoice example trivial](https://docs.oasis-open.org/ubl/UBL-2.1-JSON/v1.0/cnd02/json/UBL-Invoice-2.1-Example-Trivial.json) that can be found from UBL examples.

You can also change [ubl-rs-tester](https://github.com/hienohomma/ubl-rs-tester) to use the local version of `ubl-rs` and test builder changes there.

## License

This project is licensed under the [MIT License](https://opensource.org/licenses/MIT).

## Contributing

Pull requests, reported issues, improvements in documentation etc. are always welcome.  
Try to behave while at it.

Helpful links:

- UBL 2.1 json [specification](https://docs.oasis-open.org/ubl/UBL-2.1-JSON/v1.0/UBL-2.1-JSON-v1.0.html)
- UBL 2.1 [json schema](https://docs.oasis-open.org/ubl/UBL-2.1-JSON/v1.0/cnd02/json-schema/common/)
- UBL 2.1 [full downloadable spec](http://docs.oasis-open.org/ubl/UBL-2.1-JSON/v1.0/cnd02/UBL-2.1-JSON-v1.0-cnd02.zip)

## Contact

- Email: <opensource@hienohomma.fi>
- GitHub: [hienohomma](https://github.com/hienohomma)
