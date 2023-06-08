use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_kitty_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(KittiesModule::next_kitty_id(), 0);
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(1), *b"testtest"));
		assert_eq!(KittiesModule::next_kitty_id(), 1);
		assert!(KittiesModule::kitties(0).is_some());
		assert_eq!(KittiesModule::kitty_owner(0), Some(1));
		assert_eq!(KittiesModule::kitty_parents(0), None);

		// check for event
		mock::System::assert_last_event(
			Event::KittyCreated {
				owner: 1,
				kitty_id: 0,
				kitty: KittiesModule::kitties(0).unwrap(),
			}
			.into(),
		);
	});
}

#[test]
fn creat_kitty_insufficient_balance_fails() {
	new_test_ext().execute_with(|| {
		// user 3 is in low balance
		assert_noop!(
			KittiesModule::create_kitty(RuntimeOrigin::signed(3), *b"testtest"),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn create_kitty_overflow_fails() {
	new_test_ext().execute_with(|| {
		NextKittyId::<mock::Test>::set(KittyId::MAX);
		assert_noop!(
			KittiesModule::create_kitty(RuntimeOrigin::signed(1), *b"testtest"),
			Error::<Test>::KittiesCountOverflow
		);
	});
}

#[test]
fn breed_kitty_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(1), *b"testtest"));
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(1), *b"testtest"));
		assert_ok!(KittiesModule::breed(RuntimeOrigin::signed(1), 0, 1, *b"testtest"));

		mock::System::assert_last_event(
			Event::KittyBreed { owner: 1, kitty_id: 2, kitty: KittiesModule::kitties(2).unwrap() }
				.into(),
		);

		assert_eq!(KittiesModule::next_kitty_id(), 3);
		assert!(KittiesModule::kitties(2).is_some());
		assert_eq!(KittiesModule::kitty_owner(2), Some(1));
		assert_eq!(KittiesModule::kitty_parents(2), Some((0, 1)));
	});
}

#[test]
fn breed_kitty_insufficient_balance_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(1), *b"testtest"));
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(1), *b"testtest"));
		// user 3 is in low balance
		assert_noop!(
			KittiesModule::breed(RuntimeOrigin::signed(3), 0, 1, *b"testtest"),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn breed_kitty_same_id_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(1), *b"testtest"));
		assert_noop!(
			KittiesModule::breed(RuntimeOrigin::signed(1), 0, 0, *b"testtest"),
			Error::<Test>::SameKittyId
		);
	});
}

#[test]
fn breed_kitty_invalid_id_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(1), *b"testtest"));
		assert_noop!(
			KittiesModule::breed(RuntimeOrigin::signed(1), 0, 1, *b"testtest"),
			Error::<Test>::InvalidKittyId
		);
	});
}

#[test]
fn transfer_kitty_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(1), *b"testtest"));
		assert_ok!(KittiesModule::transfer(RuntimeOrigin::signed(1), 2, 0));
		assert_eq!(KittiesModule::kitty_owner(0), Some(2));

		mock::System::assert_last_event(
			Event::KittyTransferred { owner: 1, recipient: 2, kitty_id: 0 }.into(),
		);
	});
}

#[test]
fn transfer_kitty_not_owner_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(1), *b"testtest"));
		assert_noop!(
			KittiesModule::transfer(RuntimeOrigin::signed(2), 2, 0),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn transfer_kitty_invalid_id_fails() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			KittiesModule::transfer(RuntimeOrigin::signed(1), 2, 0),
			Error::<Test>::InvalidKittyId
		);
	});
}

#[test]
fn sell_kitty_works() {
	new_test_ext().execute_with(|| {
		// create a kitty
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(1), *b"testtest"));
		// sale the kitty
		assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(1), 0));

		// the should be in on sale list
		assert_eq!(KittiesModule::kitty_on_sale(0), Some(()));

		// sale the kitty again
		assert_noop!(KittiesModule::sale(RuntimeOrigin::signed(1), 0), Error::<Test>::AlreadyOnSale);

		mock::System::assert_last_event(Event::KittyOnSale { owner: 1, kitty_id: 0 }.into());
	});
}

#[test]
fn sell_kitty_fails() {
	new_test_ext().execute_with(|| {
		// invalid kitty id
		assert_noop!(KittiesModule::sale(RuntimeOrigin::signed(1), 0), Error::<Test>::InvalidKittyId);
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(1), *b"testtest"));
		// not the owner
		assert_noop!(KittiesModule::sale(RuntimeOrigin::signed(2), 0), Error::<Test>::NotOwner);
	});
}

#[test]
fn buy_kitty_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(1), *b"testtest"));
		assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(1), 0));

		// other user buy the kitty
		assert_ok!(KittiesModule::buy(RuntimeOrigin::signed(2), 0));

		// check new owner
		assert_eq!(KittiesModule::kitty_owner(0), Some(2));
		// the cat should not be in the on sale list
		assert_eq!(KittiesModule::kitty_on_sale(0), None);

		mock::System::assert_last_event(Event::BuyKitty { buyer: 2, owner: 1, kitty_id: 0 }.into());
	});
}

#[test]
fn buy_kitty_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(1), *b"testtest"));
		assert_noop!(KittiesModule::buy(RuntimeOrigin::signed(1), 0), Error::<Test>::AlreadyOwned);
		assert_noop!(KittiesModule::buy(RuntimeOrigin::signed(2), 0), Error::<Test>::NotOnSale);
	});
}

#[test]
fn buy_kitty_insufficient_balance_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create_kitty(RuntimeOrigin::signed(1), *b"testtest"));
		assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(1), 0));

		// user 3 is in low balance
		assert_noop!(
			KittiesModule::buy(RuntimeOrigin::signed(3), 0),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
	});
}
