#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod coin_price;
mod migrations;

#[frame_support::pallet]
pub mod pallet {
	use crate::{coin_price::CoinPriceInfo, migrations};
	use core::marker::PhantomData;
	use frame_support::{
		inherent::Vec,
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, Len, Randomness},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_core::offchain::Duration;
	use sp_io::hashing::blake2_128;
	use sp_runtime::{offchain::http, traits::AccountIdConversion};

	const ON_CHAIN_KEY: &[u8] = b"kitties_prefix";
	const STORAGE_VERSION_NUM: u16 = 2;
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(STORAGE_VERSION_NUM);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(PhantomData<T>);

	pub type KittyId = u32;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// A Kitty, represented by its unique Kitty ID and data.
	#[derive(
		Encode, Decode, Default, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen,
	)]
	pub struct Kitty {
		pub dna: [u8; 16],
		pub name: [u8; 8],
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId>;
		#[pallet::constant]
		type KittyPrice: Get<BalanceOf<Self>>;
		type PalletId: Get<PalletId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_parents)]
	pub type KittyParents<T: Config> =
		StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_on_sale)]
	pub type KittyOnSale<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An event that is emitted when a new Kitty is created.
		///
		/// It contains the following information:
		/// - The account ID of the owner of the new Kitty.
		/// - The unique ID of the new Kitty.
		/// - The data of the new Kitty.
		///
		/// Event parameters: [owner, kitty_id, kitty]
		KittyCreated {
			owner: T::AccountId,
			kitty_id: KittyId,
			kitty: Kitty,
		},
		KittyBreed {
			owner: T::AccountId,
			kitty_id: KittyId,
			kitty: Kitty,
		},
		KittyTransferred {
			owner: T::AccountId,
			recipient: T::AccountId,
			kitty_id: KittyId,
		},
		KittyOnSale {
			owner: T::AccountId,
			kitty_id: KittyId,
		},
		BuyKitty {
			buyer: T::AccountId,
			owner: T::AccountId,
			kitty_id: KittyId,
		},
		SetOffchainCoin {
			who: T::AccountId,
			coin: BoundedVec<u8, ConstU32<3>>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// KettyId overflow.
		KittiesCountOverflow,
		InvalidKittyId,
		SameKittyId,
		NotOwner,
		AlreadyOnSale,
		NoOwner,
		AlreadyOwned,
		NotOnSale,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			let weight = migrations::v2::migrate::<T>();

			// update storage version
			let current_version = Pallet::<T>::on_chain_storage_version();
			let target_version = STORAGE_VERSION_NUM;
			if current_version < target_version {
				StorageVersion::new(target_version).put::<Self>();
			}

			weight
		}

		fn offchain_worker(_block_number: T::BlockNumber) {
			let _coin = Self::fetch_coin_info();

			if let Ok(info) = Self::fetch_coin_price_info() {
				log::info!("OCW ==> coin Info: {:?}", info);
			} else {
				log::info!("OCW ==> Error while fetch coin info!");
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new Kitty.
		/// This function will create a new Kitty, save it to the Kitties map, and emit a
		/// KittyCreated event.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_kitty(origin: OriginFor<T>, name: [u8; 8]) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let kitty_id = Self::get_next_id()?;
			let kitty = Kitty { dna: Self::random_value(&who), name };

			let kitty_price = T::KittyPrice::get();
			T::Currency::transfer(
				&who,
				&Self::get_pallet_account_id(),
				kitty_price,
				ExistenceRequirement::KeepAlive,
			)?;

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);

			Self::deposit_event(Event::KittyCreated { owner: who, kitty_id, kitty });
			Ok(())
		}

		/// Breed a new Kitty.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: KittyId,
			kitty_id_2: KittyId,
			name: [u8; 8],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);
			ensure!(Kitties::<T>::contains_key(kitty_id_1), Error::<T>::InvalidKittyId);
			ensure!(Kitties::<T>::contains_key(kitty_id_2), Error::<T>::InvalidKittyId);

			let kitty_id = Self::get_next_id()?;
			let kitty_1 = Kitties::<T>::get(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
			let kitty_2 = Kitties::<T>::get(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

			let kitty =
				Kitty { dna: Self::random_value_from_two_kitty(&who, kitty_1, kitty_2), name };

			let kitty_price = T::KittyPrice::get();
			T::Currency::transfer(
				&who,
				&Self::get_pallet_account_id(),
				kitty_price,
				ExistenceRequirement::KeepAlive,
			)?;

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			KittyParents::<T>::insert(kitty_id, (kitty_id_1, kitty_id_2));

			Self::deposit_event(Event::KittyBreed { owner: who, kitty_id, kitty });

			Ok(())
		}

		/// Transfer a Kitty.
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn transfer(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			kitty_id: KittyId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(KittyOwner::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);

			let owner = KittyOwner::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			ensure!(owner == who, Error::<T>::NotOwner);

			KittyOwner::<T>::insert(kitty_id, &recipient);
			Self::deposit_event(Event::KittyTransferred { owner: who, recipient, kitty_id });
			Ok(())
		}

		/// sale a Kitty.
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn sale(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Kitties::<T>::get(kitty_id)
				.ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;

			ensure!(KittyOwner::<T>::get(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);
			ensure!(KittyOnSale::<T>::get(kitty_id).is_none(), Error::<T>::AlreadyOnSale);

			KittyOnSale::<T>::insert(kitty_id, ());
			Self::deposit_event(Event::KittyOnSale { owner: who, kitty_id });
			Ok(())
		}

		/// buy a Kitty.
		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn buy(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Kitties::<T>::get(kitty_id)
				.ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
			let owner = KittyOwner::<T>::get(kitty_id)
				.ok_or::<DispatchError>(Error::<T>::NoOwner.into())?;

			ensure!(owner != who, Error::<T>::AlreadyOwned);
			ensure!(KittyOnSale::<T>::get(kitty_id).is_some(), Error::<T>::NotOnSale);

			let kitty_price = T::KittyPrice::get();

			T::Currency::transfer(&who, &owner, kitty_price, ExistenceRequirement::KeepAlive)?;

			KittyOnSale::<T>::remove(kitty_id);
			KittyOwner::<T>::insert(kitty_id, &who);
			Self::deposit_event(Event::BuyKitty { buyer: who, owner, kitty_id });
			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn set_offchain_coin(
			origin: OriginFor<T>,
			coin: BoundedVec<u8, ConstU32<3>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			log::info!("EXTRINSIC ==> set key: {:?}", ON_CHAIN_KEY);
			log::info!("EXTRINSIC ==> set value: {:?}", sp_std::str::from_utf8(&coin).unwrap());
			sp_io::offchain_index::set(&ON_CHAIN_KEY, &coin.encode());

			Self::deposit_event(Event::SetOffchainCoin { who, coin });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get the next available Kitty ID.
		/// This function will get the next available Kitty ID, and increment the NextKittyId
		/// counter.
		fn get_next_id() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|id| -> Result<KittyId, DispatchError> {
				let current_id = *id;
				*id = id
					.checked_add(1)
					.ok_or::<DispatchError>(Error::<T>::KittiesCountOverflow.into())?;
				Ok(current_id)
			})
		}

		/// Get random value.
		pub fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}

		/// Get random value from parent kitty.
		pub fn random_value_from_two_kitty(
			owner: &T::AccountId,
			kitty_1: Kitty,
			kitty_2: Kitty,
		) -> [u8; 16] {
			let selector = Self::random_value(&owner);
			let mut data = [0u8; 16];
			for i in 0..kitty_1.dna.len() {
				data[i] = (kitty_1.dna[i] & selector[i]) | (kitty_2.dna[i] & !selector[i]);
			}
			data
		}

		fn get_pallet_account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		fn fetch_coin_price_info() -> Result<CoinPriceInfo, http::Error> {
			// prepare for send request
			let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(118_000));
			let request =
				http::Request::get("https://data.binance.com/api/v3/avgPrice?symbol=BTCUSDT");
			let pending = request
				.add_header("User-Agent", "Substrate-Offchain-Worker")
				.deadline(deadline)
				.send()
				.map_err(|_| http::Error::IoError)?;
			let response =
				pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
			if response.code != 200 {
				log::warn!("Unexpected status code: {}", response.code);
				return Err(http::Error::Unknown)
			}
			let body = response.body().collect::<Vec<u8>>();
			let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
				log::warn!("No UTF8 body");
				http::Error::Unknown
			})?;

			// parse the response str
			let coin_price: CoinPriceInfo =
				serde_json::from_str(body_str).map_err(|_| http::Error::Unknown)?;

			Ok(coin_price)
		}

		fn fetch_coin_info() -> BoundedVec<u8, ConstU32<3>> {
			let mut res = BoundedVec::<u8, ConstU32<3>>::try_from(b"BTC".to_vec()).unwrap();
			let coin_data_option: Option<Vec<u8>> = sp_io::offchain::local_storage_get(
				sp_core::offchain::StorageKind::PERSISTENT,
				&ON_CHAIN_KEY,
			);

			if let Some(coin_data_vec) = coin_data_option {
				let coin_data = BoundedVec::<u8, ConstU32<3>>::decode(&mut &coin_data_vec[..]);
				match coin_data {
					Ok(coin_data) => {
						log::info!("OCW ==> got key: {:?}", ON_CHAIN_KEY);
						log::info!(
							"OCW ==> got value: {:?}",
							sp_std::str::from_utf8(&coin_data).unwrap()
						);
						res = coin_data;
					},
					Err(_) => {
						log::error!("OCW ==> Failed to decode offchain coin data");
					},
				};
			} else {
				log::warn!("OCW ==> No coin data in offchain local storage");
			}
			res
		}
	}
}
