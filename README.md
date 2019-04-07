# Substrate Inherents Sample

A Substrate runtime sample showing how to use inherent data in your custom modules.

In this sample, we get some data from a WebAPI and store it on-chain using Substrate inherents.

## How to use inherents

Following are the steps involved in using Substrate inherents in runtime modules.

### Initial setup and prerequisites

* Create a new node runtime using the `substrate-node-new` command.
* Add the dependency `substrate-inherents` in the `cargo.toml` inside the runtime directory. This can be just like the outer Substrate packages added in there.
* Make sure that your module is declared as public in the `lib.rs`. This is to make sure that the module is accessible to the outside code for provider registration. (see Register section below)
* Add the `Inherent` parameter to your custom module definition in the `construct_runtime!` macro (in `lib.rs`).

### Define `InherentIdentifier` and `InherentType`

The `InherentIdentifier` is the unique identifier for your module's inherent data in the `InherentData` storage. This should be unique across the runtime.

The `InherentType` is the data type for the inherent data of your module.

```rust
pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"tknusd00";
pub type InherentType = u64;
```

### Define `InherentDataProvider`

Your module should define an `InherentDataProvider` type and it should implement the `ProvideInherentData` trait. This is how the consensus engine provides the inherent data to the runtime, which should be available at the block production time. The `ProvideInherentData` trait defines a function called `provide_inherent_data` where we get (or calculate) the inherent data and store it in the `InherentData` storage.

In this sample, we call a Web API to get some data and we store this data on-chain using Substrate inherents.

**Rolling Median Approach**: Assuming that 50% of the authorities are honest, a rolling median approach could be used to enhance seurity of the solution. In this sample, we collect the last 10 responses from the API in a `Vec` and then compare the most recent value with the median of those 10 values with a predefined drift value. If the difference between the median and the new value is more than the drift then we do not store that value. This could be further extended to punish the dishonest authorities.

### Register `InherentDataProvider`

To let the Substrate runtime know that there is a new `InherentDataProvider`, we need to register it. In this sample, we are using `substrate-node-template` and the provider is registered in the `/src/service.rs` file.

```rust
let reg = service.config.custom.inherent_data_providers.register_provider(inherentsample::InherentDataProvider);
```

### Implement `ProvideInherent` for your module

The final step is to implement the `ProvideInherent` trait for your module. The `ProvideInherent` trait allow the block author to store the inherent data in the module storage through an inherent extrinsics call, using the `create_inherent` function.

The `ProvideInherent` trait also has an optional function `check_inherent` which the other validators can call to verify or validate the inherent data submitted by the block author. For simplicity, we have not implemented this function in this sample.

## Note

This sample is only for showing the process or steps involved in using Substrate inherents in runtime modules. This, by no means, is intended to be a tutorial for how to build a decentralized oracle. Please do not use this code in production applications without proper review and audits.