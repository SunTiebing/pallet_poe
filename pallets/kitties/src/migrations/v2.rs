use crate::{Config, Kitties, Kitty, KittyId, Pallet};
use frame_support::{
	migration::storage_key_iter, pallet_prelude::*, storage::StoragePrefixedMap,
	traits::GetStorageVersion, weights::Weight, Blake2_128Concat,
};

#[derive(
	Encode, Decode, Default, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
pub struct V1Kitty {
	pub dna: [u8; 16],
	pub name: [u8; 4],
}

/// version 1 to version 2
pub fn migrate<T: Config>() -> Weight {
	let on_chain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::current_storage_version();

	// only works for version 1 to 2
	if on_chain_version != 1 {
		return Weight::zero()
	}
	if current_version != 2 {
		return Weight::zero()
	}

	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();

	for (index, kitty) in
		storage_key_iter::<KittyId, V1Kitty, Blake2_128Concat>(module, item).drain()
	{
		// new name rule
		let mut name = [0u8; 8];
		name[..4].copy_from_slice(&kitty.name);
		name[4..].copy_from_slice(&kitty.name);
		let new_kitty = Kitty { dna: kitty.dna, name };
		Kitties::<T>::insert(index, &new_kitty);
	}

	Weight::zero()
}
