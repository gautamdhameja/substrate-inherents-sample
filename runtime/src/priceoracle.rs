use support::{decl_module, decl_storage, StorageValue};
use rstd::prelude::*;
use system::{ensure_inherent};
use parity_codec::{Decode};
#[cfg(feature = "std")]
use inherents::{ProvideInherentData};
use inherents::{RuntimeString, InherentIdentifier, ProvideInherent, MakeFatalError, InherentData};
use runtime_io;

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"btcusd00";
pub const QUEUE_CAPACITY: u32 = 10;

//assuming the btc price could deviate by this number (USD) between consecutive values
pub const MAX_DRIFT: u64 = 100;

// `f64` would be a better InherentType in this case.
// TODO: figure out how to implement the required traits for `f64`.
pub type InherentType = u64;

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

	fn provide_inherent_data(&self, inherent_data: &mut InherentData) -> Result<(), RuntimeString> {
		use reqwest;
		use serde_json::{Value};

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
				inherent_data.put_data(INHERENT_IDENTIFIER, &price)
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
			Self::insert_into_queue(price);
			Self::get_verify_median(price);
			<Self as Store>::BtcPrice::put(price);
		}
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as InherentSample {
		pub BtcPrice get(btc_price): u64;
		PriceData get(price_data): Vec<u64>;
	}
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

impl<T: Trait> Module<T> {
	pub fn get_current_price() -> u64 {
		Self::btc_price()
	}

	// Insert into queue.
	// If capacity had been reached, remove and insert (FIFO).
	fn insert_into_queue(item: u64) {
		let mut queue = Self::price_data();
		if queue.len() as u32 == QUEUE_CAPACITY {
			queue.remove(0);
			queue.push(item);
		} else {
			queue.push(item);
		}

		runtime_io::print(queue.len() as u64);
		<Self as Store>::PriceData::put(queue);
	}

	// Calculate median of the price data in the queue.
	// Verify if the median does not shift by MAX_DRIFT with the latest price submitted as inherent.
	fn get_verify_median(price: u64) {
		let median_val = median(&mut Self::price_data());
		assert!(median_val - price <= MAX_DRIFT || price - median_val <= MAX_DRIFT, "Price value drifts more than expected!");
	}
}

fn extract_inherent_data(data: &InherentData) -> Result<InherentType, RuntimeString> {
	data.get_data::<InherentType>(&INHERENT_IDENTIFIER)
		.map_err(|_| RuntimeString::from("Invalid inherent data encoding."))?
		.ok_or_else(|| "Inherent data is not provided.".into())
}

impl<T: Trait> ProvideInherent for Module<T> {
	type Call = Call<T>;
	type Error = MakeFatalError<RuntimeString>;
	const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

	fn create_inherent(data: &InherentData) -> Option<Self::Call> {
		let data1 = extract_inherent_data(data).expect("Error in extracting inherent data.");
		Some(Call::set(data1.into()))
	}

	// TODO: Implement check_inherent.
}
