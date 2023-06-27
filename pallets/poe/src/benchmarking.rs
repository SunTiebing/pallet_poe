use crate::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	created_claim {
		let d in 0 .. T::MaxClaimLength::get();
		let claim = BoundedVec::try_from(vec![0; d as usize]).unwrap();
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller.clone()), claim.clone())
	verify {
		assert_last_event::<T>(Event::ClaimCreated(caller, claim).into())
	}

	revoke_claim {
		let d in 0 .. T::MaxClaimLength::get();
		let claim = BoundedVec::try_from(vec![0; d as usize]).unwrap();
		let caller: T::AccountId = whitelisted_caller();
		assert!(Pallet::<T>::created_claim(RawOrigin::Signed(caller.clone()).into(), claim.clone()).is_ok());
	}: _(RawOrigin::Signed(caller.clone()), claim.clone())
	verify {
		assert_last_event::<T>(Event::ClaimRevoked(caller, claim).into())
	}

	transfer_claim {
		let d in 0 .. T::MaxClaimLength::get();
		let claim = BoundedVec::try_from(vec![0; d as usize]).unwrap();
		let caller: T::AccountId = whitelisted_caller();
		let to: T::AccountId = account("to", 0, 0);
		assert!(Pallet::<T>::created_claim(RawOrigin::Signed(caller.clone()).into(), claim.clone()).is_ok());
	}: _(RawOrigin::Signed(caller.clone()), claim.clone(), to.clone())
	verify {
		assert_last_event::<T>(Event::ClaimTransferred(caller, claim, to).into())
	}

	impl_benchmark_test_suite!(PoeModule, crate::mock::new_test_ext(), crate::mock::Test);
}
