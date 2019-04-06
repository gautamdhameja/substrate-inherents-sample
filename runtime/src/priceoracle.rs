use support::{decl_module, decl_storage, StorageValue};
use rstd::prelude::*;
use system::{ensure_inherent};
use parity_codec::{Decode};
#[cfg(feature = "std")]
use inherents::{ProvideInherentData};
use inherents::{RuntimeString, InherentIdentifier, ProvideInherent, MakeFatalError, InherentData};

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"btcusd00";

// `f64` would be a better InherentType in this case.
// TODO: figure out how to implement the required traits for `f64`.
pub type InherentType = Vec<u64>;

pub trait BtcPriceInherentData {
	fn btcusd_inherent_data(&self) -> Result<InherentType, RuntimeString>;
}

impl BtcPriceInherentData for InherentData {
	fn btcusd_inherent_data(&self) -> Result<InherentType, RuntimeString> {
		self.get_data(&INHERENT_IDENTIFIER)
			.and_then(|r| r.ok_or_else(|| "Inherent data not found".into()))
	}
}

#[cfg(feature = "std")]
pub struct InherentDataProvider;

#[cfg(feature = "std")]
impl ProvideInherentData for InherentDataProvider {
	fn inherent_identifier(&self) -> &'static InherentIdentifier {
		&INHERENT_IDENTIFIER
	}

	// Is there a way to access the module's storage from here?
	// At the moment, we are initializing the queue if it doesn't already exist and
	// storing it as a `Vec` in the inherent data storage.
	// If we could access the module storage, should we store the queue there?
	fn provide_inherent_data(&self, inherent_data: &mut InherentData) -> Result<(), RuntimeString> {
		use reqwest;
		use serde_json::{Value};
		use std::collections::VecDeque;
		let mut queue: VecDeque<u64>;

		let data = inherent_data.btcusd_inherent_data();

		match data {
			Err(_) => {
				queue = VecDeque::with_capacity(10);
			 }
			_ => {
				println!("{:?}", data.clone().unwrap());
				queue = data.unwrap().into();
			 }
		}

		println!("{:?}", queue.capacity());

		reqwest::get("https://api.coindesk.com/v1/bpi/currentprice.json")
		.map_err(|_| {
				"Could not get BTC price from API".into()
		}).and_then(|mut resp| {
			resp.text()
			.map_err(|_| {
				"Could not get response body".into()
			}).and_then(|body| {
				let v: Value = serde_json::from_str(&body).unwrap();
				let v_price = &v["bpi"]["USD"]["rate_float"];
				let price: u64 = v_price.as_f64().unwrap().round() as u64;
				
				// if the queue has reached max capacity, remove the first inserted element.
				// Add the new price to the end.
				// This makes sure that we have a fixed capacity rotating window.
				
				// TODO: fix this
				if queue.len() == queue.capacity() {
					queue.pop_front();
				}

				queue.push_back(price);
				println!("{:?}", queue.capacity());
				println!("{:?}", queue.len());
				println!("{:?}", queue);
				let vec = Vec::from(queue);
				inherent_data.put_data(INHERENT_IDENTIFIER, &vec)
			})
		})
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		RuntimeString::decode(&mut &error[..]).map(Into::into)
	}
}

/// The module configuration trait
pub trait Trait: system::Trait { }

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn set(origin, price: u64) {
			ensure_inherent(origin)?;

			// TODO: Add assetion logic to check the price is within a threshold.

			<Self as Store>::BtcPrice::put(price);
		}
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as InherentSample {
		pub BtcPrice get(btc_price): u64;
		PriceData: Option<Vec<u64>>;
	}
}

impl<T: Trait> Module<T> {
	pub fn get_current_price() -> u64 {
		Self::btc_price()
	}
}

fn extract_inherent_data(data: &InherentData) -> Result<InherentType, RuntimeString> {
	data.get_data::<InherentType>(&INHERENT_IDENTIFIER)
		.map_err(|_| RuntimeString::from("Invalid inherent data encoding."))?
		.ok_or_else(|| "Inherent data is not provided.".into())
}

// Following two functions (mean and median) taken from:
// https://gist.github.com/ayoisaiah/185fec1ca98ce44fca1308753182ff2b

fn mean(numbers: &Vec<u64>) -> u64 {
    let sum: u64 = numbers.iter().sum();
    sum as u64 / numbers.len() as u64
}

fn median(numbers: &mut Vec<u64>) -> u64 {
    numbers.sort();
    let mid = numbers.len() / 2;
    if numbers.len() % 2 == 0 {
        mean(&vec![numbers[mid - 1], numbers[mid]]) as u64
    } else {
        numbers[mid]
    }
}

impl<T: Trait> ProvideInherent for Module<T> {
	type Call = Call<T>;
	type Error = MakeFatalError<RuntimeString>;
	const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

	fn create_inherent(data: &InherentData) -> Option<Self::Call> {
		let mut data1 = extract_inherent_data(data).expect("Error in extracting inherent data.");
		let median_val = median(data1.as_mut());
		Some(Call::set(median_val.into()))
	}

	// TODO: Implement check_inherent.
}
