use crate as pallet_kitties;
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64},
	PalletId,
};
use frame_system as system;
use pallet_balances;
use pallet_randomness_collective_flip;
use sp_core::{ConstU128, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		KittiesModule: pallet_kitties,
		Randomness: pallet_randomness_collective_flip,
		Balances: pallet_balances,
	}
);

/// Balance of an account.
pub type Balance = u128;

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

/// Existential deposit.
pub const EXISTENTIAL_DEPOSIT: u128 = 500;

parameter_types! {
	pub KittyPalletId: PalletId = PalletId(*b"py/kitty");
	pub KittyPrice: Balance = EXISTENTIAL_DEPOSIT * 10;
}

impl pallet_kitties::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Randomness = Randomness;
	type Currency = Balances;
	type KittyPrice = KittyPrice;
	type PalletId = KittyPalletId;
}

impl pallet_randomness_collective_flip::Config for Test {}

impl pallet_balances::Config for Test {
	/// The type for recording an account's balance.
	type Balance = Balance;
	type DustRemoval = ();
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	// set initial balance
	pallet_balances::GenesisConfig::<Test> { balances: vec![(1, 10_000_000), (2, 10_000_000)] }
		.assimilate_storage(&mut ext)
		.unwrap();
	let mut ext: sp_io::TestExternalities = ext.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
