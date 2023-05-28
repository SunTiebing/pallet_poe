#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use core::marker::PhantomData;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
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
	}

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitteies)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

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
		KittyCreated { owner: T::AccountId, kitty_id: KittyId, kitty: Kitty },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// KettyId overflow.
		InvalidKittyId,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new Kitty.
		/// This function will create a new Kitty, save it to the Kitties map, and emit a KittyCreated event.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let kitty_id = Self::get_next_id()?;
			let kitty = Kitty::default();

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);

			Self::deposit_event(Event::KittyCreated { owner: who, kitty_id, kitty });
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
					id.checked_add(1).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
				Ok(current_id)
			})
		}
	}
}
