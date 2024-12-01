# vcg-auction

A [Vickrey-Clarke-Groves
auction](https://en.wikipedia.org/wiki/Vickrey%E2%80%93Clarke%E2%80%93Groves_auction)
library.

The VCG Auction is the most economically efficient auction format. It
encourages truthful bidding by only charging bidders their externality—the
value of the best alternative outcome had the bidder not participated. In its
exact form, it’s a combinatorial auction that is computationally hard to
compute for large problem sizes. Nevertheless, small problems are certainly
possible to compute by exhaustive search, and that's what this library does.

There's many computationally-feasible simulations of the VCG auction that can
scale better for practical use cases, like the Simultaneous Ascending Auction
used for wireless spectrum allocation.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
