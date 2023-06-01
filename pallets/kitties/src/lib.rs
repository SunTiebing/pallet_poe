#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use core::marker::PhantomData;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Len, Randomness};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use std::default::Default;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	pub type KittyId = u32;

	/// A Kitty, represented by its unique Kitty ID and data.
	#[derive(
		Encode, Decode, Default, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen,
	)]
	pub struct Kitty(pub [u8; 16]);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
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
	}

	#[pallet::error]
	pub enum Error<T> {
		/// KettyId overflow.
		KittiesCountOverflow,
		InvalidKittyId,
		SameKittyId,
		NotOwner,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new Kitty.
		/// This function will create a new Kitty, save it to the Kitties map, and emit a KittyCreated event.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let kitty_id = Self::get_next_id()?;
			let kitty = Kitty(Self::random_value(&who));

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);

			Self::deposit_event(Event::KittyCreated { owner: who, kitty_id, kitty });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: KittyId,
			kitty_id_2: KittyId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);
			ensure!(Kitties::<T>::contains_key(kitty_id_1), Error::<T>::InvalidKittyId);
			ensure!(Kitties::<T>::contains_key(kitty_id_2), Error::<T>::InvalidKittyId);

			let kitty_id = Self::get_next_id()?;
			let kitty_1 = Kitties::<T>::get(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
			let kitty_2 = Kitties::<T>::get(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

			let selector = Self::random_value(&who);
			let mut data = [0u8; 16];
			for i in 0..kitty_1.0.len() {
				data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & !selector[i]);
			}
			let kitty = Kitty(data);

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			KittyParents::<T>::insert(kitty_id, (kitty_id_1, kitty_id_2));

			Self::deposit_event(Event::KittyBreed { owner: who, kitty_id, kitty });

			Ok(())
		}

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
	}

	/// Get the next available Kitty ID.
	/// This function will get the next available Kitty ID, and increment the NextKittyId counter.
	impl<T: Config> Pallet<T> {
		fn get_next_id() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|id| -> Result<KittyId, DispatchError> {
				let current_id = *id;
				*id =
					id.checked_add(1).ok_or::<DispatchError>(Error::<T>::KittiesCountOverflow.into())?;
				Ok(current_id)
			})
		}

		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}
	}
}
