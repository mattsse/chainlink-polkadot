use super::*;

use frame_support::weights::Weight;
use frame_support::{assert_noop, assert_ok, impl_outer_origin, parameter_types};
use sp_core::H256;

use frame_system as system;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

impl_outer_origin! {
	pub enum Origin for Test {}
}

// Configure a mock runtime to test the pallet.

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

type AccountId = u64;
type BlockNumber = u64;
impl system::Trait for Test {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}
type System = frame_system::Module<Test>;

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

type Balance = u64;
impl pallet_balances::Trait for Test {
	type MaxLocks = ();
	type Balance = Balance;
	type Event = ();
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}
type Balances = pallet_balances::Module<Test>;

const MIN_RESERVE: u64 = 100;

parameter_types! {
	pub const FeedModuleId: ModuleId = ModuleId(*b"linkfeed");
	pub const MinimumReserve: u64 = MIN_RESERVE;
	pub const StringLimit: u32 = 15;
	pub const OracleLimit: u32 = 10;
	pub const FeedLimit: u16 = 10;
	pub const PruningWindow: u32 = 3;
}

type FeedId = u16;
type Value = u64;

impl Trait for Test {
	type Event = ();
	type FeedId = FeedId;
	type Value = Value;
	type Currency = Balances;
	type ModuleId = FeedModuleId;
	type MinimumReserve = MinimumReserve;
	type StringLimit = StringLimit;
	type OracleCountLimit = OracleLimit;
	type FeedLimit = FeedLimit;
	type PruningWindow = PruningWindow;
	type WeightInfo = ();
}
type ChainlinkFeed = crate::Module<Test>;

#[derive(Debug, Default)]
struct FeedBuilder {
	owner: Option<AccountId>,
	payment: Option<Balance>,
	timeout: Option<BlockNumber>,
	value_bounds: Option<(Value, Value)>,
	min_submissions: Option<u32>,
	description: Option<Vec<u8>>,
	restart_delay: Option<RoundId>,
	oracles: Option<Vec<(AccountId, AccountId)>>,
}

impl FeedBuilder {
	fn new() -> Self {
		Self::default()
	}

	fn owner(mut self, o: AccountId) -> Self {
		self.owner = Some(o);
		self
	}

	fn payment(mut self, p: Balance) -> Self {
		self.payment = Some(p);
		self
	}

	fn timeout(mut self, t: BlockNumber) -> Self {
		self.timeout = Some(t);
		self
	}

	fn value_bounds(mut self, min: Value, max: Value) -> Self {
		self.value_bounds = Some((min, max));
		self
	}

	fn min_submissions(mut self, m: u32) -> Self {
		self.min_submissions = Some(m);
		self
	}

	fn description(mut self, d: Vec<u8>) -> Self {
		self.description = Some(d);
		self
	}

	fn restart_delay(mut self, d: RoundId) -> Self {
		self.restart_delay = Some(d);
		self
	}

	fn oracles(mut self, o: Vec<(AccountId, AccountId)>) -> Self {
		self.oracles = Some(o);
		self
	}

	fn build_and_store(self) -> DispatchResultWithPostInfo {
		let owner = Origin::signed(self.owner.unwrap_or(1));
		let payment = self.payment.unwrap_or(20);
		let timeout = self.timeout.unwrap_or(1);
		let value_bounds = self.value_bounds.unwrap_or((1, 1_000));
		let min_submissions = self.min_submissions.unwrap_or(2);
		let decimals = 5;
		let description = self.description.unwrap_or(b"desc".to_vec());
		let oracles = self.oracles.unwrap_or(vec![(2, 4), (3, 4), (4, 4)]);
		let restart_delay = self
			.restart_delay
			.unwrap_or(oracles.len().saturating_sub(1) as u32);
		ChainlinkFeed::create_feed(
			owner,
			payment,
			timeout,
			value_bounds,
			min_submissions,
			decimals,
			description,
			restart_delay,
			oracles,
		)
	}
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	let module_account: AccountId = FeedModuleId::get().into_account();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(module_account, 100 * MIN_RESERVE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	crate::GenesisConfig::<Test> {
		pallet_admin: module_account,
		feed_creators: vec![1],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	t.into()
}

#[test]
fn feed_creation_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(ChainlinkFeed::create_feed(
			Origin::signed(1),
			20,
			10,
			(10, 1_000),
			3,
			5,
			b"desc".to_vec(),
			2,
			vec![(1, 4), (2, 4), (3, 4)],
		));
	});
}

#[test]
fn feed_creation_failure_cases() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			FeedBuilder::new().owner(123).build_and_store(),
			Error::<Test>::NotFeedCreator
		);
		assert_noop!(
			FeedBuilder::new()
				.description(b"waaaaaaaaaaaaaaaaay too long".to_vec())
				.build_and_store(),
			Error::<Test>::DescriptionTooLong
		);
		let too_many_oracles = (0..(OracleLimit::get() + 1))
			.into_iter()
			.map(|i| (i as u64, i as u64))
			.collect();
		assert_noop!(
			FeedBuilder::new()
				.oracles(too_many_oracles)
				.build_and_store(),
			Error::<Test>::OraclesLimitExceeded
		);
		assert_noop!(
			FeedBuilder::new()
				.min_submissions(3)
				.oracles(vec![(1, 2)])
				.build_and_store(),
			Error::<Test>::WrongBounds
		);
		assert_noop!(
			FeedBuilder::new()
				.min_submissions(0)
				.oracles(vec![(1, 2)])
				.build_and_store(),
			Error::<Test>::WrongBounds
		);
		assert_noop!(
			FeedBuilder::new()
				.oracles(vec![(1, 2), (2, 3), (3, 4)])
				.restart_delay(3)
				.build_and_store(),
			Error::<Test>::DelayNotBelowCount
		);

		for _feed in 0..FeedLimit::get() {
			assert_ok!(FeedBuilder::new().build_and_store());
		}
		assert_noop!(
			FeedBuilder::new().build_and_store(),
			Error::<Test>::FeedLimitReached
		);
	});
}
#[test]
fn submit_should_work() {
	new_test_ext().execute_with(|| {
		let payment = 20;
		let timeout = 10;
		let min_submissions = 2;
		let oracles = vec![(1, 4), (2, 4), (3, 4)];
		let submission_count_bounds = (min_submissions, oracles.len() as u32);
		assert_ok!(FeedBuilder::new()
			.payment(payment)
			.timeout(timeout)
			.min_submissions(min_submissions)
			.oracles(oracles)
			.build_and_store());

		let feed_id = 0;
		let round_id = 1;
		let oracle = 2;
		let submission = 42;
		assert_ok!(ChainlinkFeed::submit(
			Origin::signed(oracle),
			feed_id,
			round_id,
			submission
		));
		let second_oracle = 3;
		assert_ok!(ChainlinkFeed::submit(
			Origin::signed(second_oracle),
			feed_id,
			round_id,
			submission
		));
		let round = ChainlinkFeed::round(feed_id, round_id).expect("first round should be present");
		assert_eq!(
			round,
			Round {
				started_at: 0,
				answer: Some(submission),
				updated_at: Some(0),
				answered_in_round: Some(1),
			}
		);
		let details = ChainlinkFeed::round_details(feed_id, round_id)
			.expect("details for first round should be present");
		assert_eq!(
			details,
			RoundDetails {
				submissions: vec![submission, submission],
				submission_count_bounds,
				payment,
				timeout,
			}
		);
		let oracle_status =
			ChainlinkFeed::oracle_status(feed_id, oracle).expect("oracle status should be present");
		assert_eq!(oracle_status.latest_submission, Some(submission));
	});
}

#[test]
fn details_are_cleared() {
	new_test_ext().execute_with(|| {
		let payment = 20;
		let timeout = 10;
		let min_submissions = 2;
		let oracle = 2;
		let snd_oracle = 3;
		let oracles = vec![(1, 4), (oracle, 4), (snd_oracle, 4)];
		assert_ok!(FeedBuilder::new()
			.payment(payment)
			.timeout(timeout)
			.min_submissions(min_submissions)
			.oracles(oracles)
			.build_and_store());

		let feed_id = 0;
		{
			// round 1
			let r = 1;
			let submission = 42;
			let answer = submission;
			assert_ok!(ChainlinkFeed::submit(
				Origin::signed(oracle),
				feed_id,
				r,
				submission
			));
			assert_ok!(ChainlinkFeed::submit(
				Origin::signed(snd_oracle),
				feed_id,
				r,
				submission
			));
			let round = ChainlinkFeed::round(feed_id, r).unwrap();
			assert_eq!(round.answer, Some(answer));
			let details = ChainlinkFeed::round_details(feed_id, r).unwrap();
			assert_eq!(details.submissions, vec![submission, submission]);
			let oracle_status = ChainlinkFeed::oracle_status(feed_id, oracle).unwrap();
			assert_eq!(oracle_status.latest_submission, Some(submission));
		}
		{
			// round 2
			let r = 2;
			let submission = 21;
			let answer = submission;
			// switch the order because `oracle` is not allowed
			// to start a new round
			assert_ok!(ChainlinkFeed::submit(
				Origin::signed(snd_oracle),
				feed_id,
				r,
				submission
			));
			assert_ok!(ChainlinkFeed::submit(
				Origin::signed(oracle),
				feed_id,
				r,
				submission
			));
			let round = ChainlinkFeed::round(feed_id, r).unwrap();
			assert_eq!(round.answer, Some(answer));
			let details = ChainlinkFeed::round_details(feed_id, r).unwrap();
			assert_eq!(details.submissions, vec![submission, submission]);
			let oracle_status = ChainlinkFeed::oracle_status(feed_id, oracle).unwrap();
			assert_eq!(oracle_status.latest_submission, Some(submission));
			// old round details should be gone
			assert_eq!(ChainlinkFeed::round_details(feed_id, 1), None);
		}
	});
}

#[test]
fn submit_failure_cases() {
	new_test_ext().execute_with(|| {
		let oracle = 2;
		let oracles = vec![(1, 4), (oracle, 4), (3, 4)];
		assert_ok!(FeedBuilder::new()
			.value_bounds(1, 100)
			.oracles(oracles)
			.build_and_store());

		let feed_id = 0;
		let no_feed = 1234;
		let round_id = 1;
		let submission = 42;
		assert_noop!(
			ChainlinkFeed::submit(Origin::signed(oracle), no_feed, round_id, submission),
			Error::<Test>::FeedNotFound
		);
		let not_oracle = 1337;
		assert_noop!(
			ChainlinkFeed::submit(Origin::signed(not_oracle), feed_id, round_id, submission),
			Error::<Test>::NotOracle
		);
		let invalid_round = 1337;
		assert_noop!(
			ChainlinkFeed::submit(Origin::signed(oracle), feed_id, invalid_round, submission),
			Error::<Test>::InvalidRound
		);
		let low_value = 0;
		assert_noop!(
			ChainlinkFeed::submit(Origin::signed(oracle), feed_id, round_id, low_value,),
			Error::<Test>::SubmissionBelowMinimum
		);
		let high_value = 13377331;
		assert_noop!(
			ChainlinkFeed::submit(Origin::signed(oracle), feed_id, round_id, high_value,),
			Error::<Test>::SubmissionAboveMaximum
		);
	});
}

#[test]
fn change_oracles_should_work() {
	new_test_ext().execute_with(|| {
		let oracle = 2;
		let admin = 4;
		let initial_oracles = vec![(1, admin), (oracle, admin), (3, admin)];
		assert_ok!(FeedBuilder::new()
			.oracles(initial_oracles.clone())
			.min_submissions(1)
			.build_and_store());
		for (o, _a) in initial_oracles.iter() {
			assert!(
				ChainlinkFeed::oracle(o).is_some(),
				"oracle should be present"
			);
		}
		let feed_id = 0;
		let owner = 1;
		let feed = ChainlinkFeed::feed_config(feed_id).expect("feed should be there");
		assert_eq!(feed.oracle_count, 3);

		let round = 1;
		let submission = 42;
		assert_ok!(ChainlinkFeed::submit(
			Origin::signed(oracle),
			feed_id,
			round,
			submission
		));

		let to_disable: Vec<u64> = initial_oracles
			.iter()
			.cloned()
			.take(2)
			.map(|(o, _a)| o)
			.collect();
		let to_add = vec![(6, 9), (7, 9), (8, 9)];
		// failing cases
		assert_noop!(
			ChainlinkFeed::change_oracles(
				Origin::signed(owner),
				123,
				to_disable.clone(),
				to_add.clone(),
			),
			Error::<Test>::FeedNotFound
		);
		assert_noop!(
			ChainlinkFeed::change_oracles(
				Origin::signed(123),
				feed_id,
				to_disable.clone(),
				to_add.clone(),
			),
			Error::<Test>::NotFeedOwner
		);
		// we cannot disable the oracles before adding them
		let cannot_disable = to_add.iter().cloned().take(2).map(|(o, _a)| o).collect();
		assert_noop!(
			ChainlinkFeed::change_oracles(
				Origin::signed(owner),
				feed_id,
				cannot_disable,
				to_add.clone(),
			),
			Error::<Test>::OracleNotFound
		);
		let too_many_oracles = (0..(OracleLimit::get() + 1))
			.into_iter()
			.map(|i| (i as u64, i as u64))
			.collect();
		assert_noop!(
			ChainlinkFeed::change_oracles(
				Origin::signed(owner),
				feed_id,
				to_disable.clone(),
				too_many_oracles,
			),
			Error::<Test>::OraclesLimitExceeded
		);
		let changed_admin = initial_oracles
			.iter()
			.cloned()
			.map(|(o, _a)| (o, 33))
			.collect();
		assert_noop!(
			ChainlinkFeed::change_oracles(
				Origin::signed(owner),
				feed_id,
				to_disable.clone(),
				changed_admin,
			),
			Error::<Test>::OwnerCannotChangeAdmin
		);
		let many_duplicates: Vec<AccountId> = initial_oracles.iter().cloned().chain(initial_oracles.iter().cloned()).map(|(o, _a)| o).collect();
		assert_noop!(
			ChainlinkFeed::change_oracles(
				Origin::signed(owner),
				feed_id,
				many_duplicates.clone(),
				to_add.clone(),
			),
			Error::<Test>::NotEnoughOracles
		);
		let duplicates = vec![1, 1, 1];
		assert_noop!(
			ChainlinkFeed::change_oracles(
				Origin::signed(owner),
				feed_id,
				duplicates.clone(),
				to_add.clone(),
			),
			Error::<Test>::OracleDisabled
		);

		{
			assert_ok!(ChainlinkFeed::feed_mut(feed_id).unwrap().request_new_round(AccountId::default()));
		}
		// successfully change oracles
		assert_ok!(ChainlinkFeed::change_oracles(
			Origin::signed(owner),
			feed_id,
			to_disable.clone(),
			to_add.clone(),
		));

		// we cannot disable the same oracles a second time
		assert_noop!(
			ChainlinkFeed::change_oracles(
				Origin::signed(owner),
				feed_id,
				to_disable.clone(),
				to_add.clone(),
			),
			Error::<Test>::OracleDisabled
		);
		assert_noop!(
			ChainlinkFeed::change_oracles(Origin::signed(owner), feed_id, vec![], to_add.clone(),),
			Error::<Test>::AlreadyEnabled
		);

		let feed = ChainlinkFeed::feed_config(feed_id).expect("feed should be there");
		assert_eq!(feed.oracle_count, 4);
		assert_eq!(Oracles::<Test>::iter().count(), 6);
		assert_eq!(OracleStatuses::<Test>::iter().count(), 6);
		for o in to_disable.iter() {
			assert!(
				ChainlinkFeed::oracle_status(feed_id, o)
					.unwrap()
					.ending_round
					.is_some(),
				"oracle should be disabled"
			);
		}
		for (o, _a) in to_add.iter() {
			assert!(
				ChainlinkFeed::oracle(o).is_some(),
				"oracle should be present"
			);
		}
		assert_ok!(ChainlinkFeed::change_oracles(
			Origin::signed(owner),
			feed_id,
			vec![],
			vec![(oracle, admin)],
		));
		let expected_status = OracleStatus {
			starting_round: 2,
			ending_round: None,
			last_reported_round: Some(1),
			last_started_round: Some(1),
			latest_submission: Some(submission),
		};
		assert_eq!(
			ChainlinkFeed::oracle_status(feed_id, oracle),
			Some(expected_status)
		);
	});
}

#[test]
fn update_future_rounds_should_work() {
	new_test_ext().execute_with(|| {
		let old_payment = 20;
		let old_timeout = 10;
		let old_min = 3;
		let oracles = vec![(1, 4), (2, 4), (3, 4)];
		assert_ok!(FeedBuilder::new()
			.payment(old_payment)
			.timeout(old_timeout)
			.min_submissions(old_min)
			.oracles(oracles.clone())
			.build_and_store());
		let feed_id = 0;
		let feed = ChainlinkFeed::feed_config(feed_id).expect("feed should be there");
		assert_eq!(feed.payment, old_payment);

		let owner = 1;
		let new_payment = 30;
		let new_min = 3;
		let new_max = 3;
		let new_delay = 1;
		let new_timeout = 5;
		// failure cases
		assert_noop!(
			ChainlinkFeed::update_future_rounds(
				Origin::signed(owner),
				5,
				new_payment,
				(new_min, new_max),
				new_delay,
				new_timeout,
			),
			Error::<Test>::FeedNotFound
		);
		assert_noop!(
			ChainlinkFeed::update_future_rounds(
				Origin::signed(123),
				feed_id,
				new_payment,
				(new_min, new_max),
				new_delay,
				new_timeout,
			),
			Error::<Test>::NotFeedOwner
		);
		assert_noop!(
			ChainlinkFeed::update_future_rounds(
				Origin::signed(owner),
				feed_id,
				new_payment,
				(new_max + 1, new_max),
				new_delay,
				new_timeout,
			),
			Error::<Test>::WrongBounds
		);
		assert_noop!(
			ChainlinkFeed::update_future_rounds(
				Origin::signed(owner),
				feed_id,
				new_payment,
				(new_min, oracles.len() as u32 + 1),
				new_delay,
				new_timeout,
			),
			Error::<Test>::MaxExceededTotal
		);
		assert_noop!(
			ChainlinkFeed::update_future_rounds(
				Origin::signed(owner),
				feed_id,
				new_payment,
				(new_min, new_max),
				oracles.len() as RoundId,
				new_timeout,
			),
			Error::<Test>::DelayNotBelowCount
		);
		assert_noop!(
			ChainlinkFeed::update_future_rounds(
				Origin::signed(owner),
				feed_id,
				new_payment,
				(0, new_max),
				new_delay,
				new_timeout,
			),
			Error::<Test>::WrongBounds
		);

		// successful update
		assert_ok!(ChainlinkFeed::update_future_rounds(
			Origin::signed(owner),
			feed_id,
			new_payment,
			(new_min, new_max),
			new_delay,
			new_timeout,
		));

		let feed_id = 0;
		let feed = ChainlinkFeed::feed_config(feed_id).expect("feed should be there");
		assert_eq!(feed.payment, new_payment);
	});
}

#[test]
fn admin_transfer_should_work() {
	new_test_ext().execute_with(|| {
		let oracle = 1;
		let old_admin = 2;
		assert_ok!(FeedBuilder::new()
			.min_submissions(1)
			.restart_delay(0)
			.oracles(vec![(oracle, old_admin)])
			.build_and_store());

		let new_admin = 42;
		assert_noop!(ChainlinkFeed::transfer_admin(
			Origin::signed(old_admin),
			123,
			new_admin
		), Error::<Test>::OracleNotFound);
		assert_noop!(ChainlinkFeed::transfer_admin(
			Origin::signed(123),
			oracle,
			new_admin
		), Error::<Test>::NotAdmin);
		assert_ok!(ChainlinkFeed::transfer_admin(
			Origin::signed(old_admin),
			oracle,
			new_admin
		));
		let oracle_meta = ChainlinkFeed::oracle(oracle).expect("oracle should be present");
		assert_eq!(oracle_meta.pending_admin, Some(new_admin));
		assert_noop!(ChainlinkFeed::accept_admin(
			Origin::signed(new_admin),
			123,
		), Error::<Test>::OracleNotFound);
		assert_noop!(ChainlinkFeed::accept_admin(
			Origin::signed(123),
			oracle
		), Error::<Test>::NotPendingAdmin);
		assert_ok!(ChainlinkFeed::accept_admin(
			Origin::signed(new_admin),
			oracle
		));
		let oracle_meta = ChainlinkFeed::oracle(oracle).expect("oracle should be present");
		assert_eq!(oracle_meta.pending_admin, None);
		assert_eq!(oracle_meta.admin, new_admin);
	});
}

#[test]
fn request_new_round_should_work() {
	new_test_ext().execute_with(|| {
		let owner = 1;
		let payment = 20;
		let timeout = 10;
		let min_submissions = 2;
		let oracles = vec![(1, 4), (2, 4), (3, 4)];
		let submission_count_bounds = (min_submissions, oracles.len() as u32);
		assert_ok!(FeedBuilder::new()
			.owner(owner)
			.payment(payment)
			.timeout(timeout)
			.min_submissions(min_submissions)
			.oracles(oracles)
			.build_and_store());

		let feed_id = 0;
		let requester = 22;
		let snd_requester = 33;
		let delay = 4;
		assert_ok!(ChainlinkFeed::set_requester(
			Origin::signed(owner),
			feed_id,
			requester,
			delay
		));
		assert_ok!(ChainlinkFeed::set_requester(
			Origin::signed(owner),
			feed_id,
			snd_requester,
			delay
		));
		let requester_meta =
			ChainlinkFeed::requester(feed_id, requester).expect("requester should be present");
		assert_eq!(
			requester_meta,
			Requester {
				delay,
				last_started_round: None
			}
		);
		// failure cases
		assert_noop!(ChainlinkFeed::request_new_round(
			Origin::signed(123),
			feed_id
		), Error::<Test>::NotAuthorizedRequester);
		// non existent feed is not present but also not authorized
		assert_noop!(ChainlinkFeed::request_new_round(
			Origin::signed(requester),
			123
		), Error::<Test>::NotAuthorizedRequester);

		// actually request new round
		assert_ok!(ChainlinkFeed::request_new_round(
			Origin::signed(requester),
			feed_id
		));
		// need to wait `delay` rounds before requesting again
		assert_noop!(ChainlinkFeed::request_new_round(
			Origin::signed(requester),
			feed_id
		), Error::<Test>::CannotRequestRoundYet);
		// round does not have data and is not timed out
		// --> not supersedable
		assert_noop!(ChainlinkFeed::request_new_round(
			Origin::signed(snd_requester),
			feed_id
		), Error::<Test>::RoundNotSupersedable);

		let round_id = 1;
		let round = ChainlinkFeed::round(feed_id, round_id).expect("first round should be present");
		assert_eq!(
			round,
			Round {
				started_at: 0,
				..Default::default()
			}
		);
		let details = ChainlinkFeed::round_details(feed_id, round_id)
			.expect("details for first round should be present");
		assert_eq!(
			details,
			RoundDetails {
				submissions: Vec::new(),
				submission_count_bounds,
				payment,
				timeout,
			}
		);
		let requester_meta =
			ChainlinkFeed::requester(feed_id, requester).expect("requester should be present");
		assert_eq!(
			requester_meta,
			Requester {
				delay,
				last_started_round: Some(1)
			}
		);
	});
}

#[test]
fn requester_permissions() {
	new_test_ext().execute_with(|| {
		let owner = 1;
		assert_ok!(FeedBuilder::new().owner(owner).build_and_store());

		let feed_id = 0;
		let requester = 22;
		let delay = 4;
		// failure cases
		assert_noop!(
			ChainlinkFeed::set_requester(Origin::signed(owner), 123, requester, delay),
			Error::<Test>::FeedNotFound
		);
		assert_noop!(
			ChainlinkFeed::set_requester(Origin::signed(123), feed_id, requester, delay),
			Error::<Test>::NotFeedOwner
		);
		// actually set the requester
		assert_ok!(ChainlinkFeed::set_requester(
			Origin::signed(owner),
			feed_id,
			requester,
			delay
		));
		let requester_meta =
			ChainlinkFeed::requester(feed_id, requester).expect("requester should be present");
		assert_eq!(
			requester_meta,
			Requester {
				delay,
				last_started_round: None
			}
		);
		// failure cases
		assert_noop!(
			ChainlinkFeed::remove_requester(Origin::signed(owner), 123, requester),
			Error::<Test>::FeedNotFound
		);
		assert_noop!(
			ChainlinkFeed::remove_requester(Origin::signed(123), feed_id, requester),
			Error::<Test>::NotFeedOwner
		);
		assert_noop!(
			ChainlinkFeed::remove_requester(Origin::signed(owner), feed_id, 123),
			Error::<Test>::RequesterNotFound
		);
		// actually remove the requester
		assert_ok!(ChainlinkFeed::remove_requester(
			Origin::signed(owner),
			feed_id,
			requester
		));
		assert_eq!(ChainlinkFeed::requester(feed_id, requester), None);
	});
}

#[test]
fn transfer_ownership_should_work() {
	new_test_ext().execute_with(|| {
		let old_owner = 1;
		assert_ok!(FeedBuilder::new().owner(old_owner).build_and_store());

		let feed_id = 0;
		let new_owner = 42;
		assert_noop!(
			ChainlinkFeed::transfer_ownership(Origin::signed(old_owner), 5, new_owner),
			Error::<Test>::FeedNotFound
		);
		assert_noop!(
			ChainlinkFeed::transfer_ownership(Origin::signed(23), feed_id, new_owner),
			Error::<Test>::NotFeedOwner
		);
		assert_ok!(ChainlinkFeed::transfer_ownership(
			Origin::signed(old_owner),
			feed_id,
			new_owner
		));
		assert_ok!(ChainlinkFeed::transfer_ownership(
			Origin::signed(old_owner),
			feed_id,
			new_owner
		));
		let feed = ChainlinkFeed::feed_config(feed_id).expect("feed should be there");
		assert_eq!(feed.pending_owner, Some(new_owner));
		assert_noop!(
			ChainlinkFeed::accept_ownership(Origin::signed(new_owner), 123),
			Error::<Test>::FeedNotFound
		);
		assert_noop!(
			ChainlinkFeed::accept_ownership(Origin::signed(old_owner), feed_id),
			Error::<Test>::NotPendingOwner
		);
		assert_ok!(ChainlinkFeed::accept_ownership(
			Origin::signed(new_owner),
			feed_id
		));
		let feed = ChainlinkFeed::feed_config(feed_id).expect("feed should be there");
		assert_eq!(feed.pending_owner, None);
		assert_eq!(feed.owner, new_owner);
	});
}

#[test]
fn feed_oracle_trait_should_work() {
	new_test_ext().execute_with(|| {
		let oracle = 2;
		let second_oracle = 3;
		assert_ok!(FeedBuilder::new()
			.oracles(vec![(oracle, 4), (second_oracle, 4)])
			.build_and_store());

		let feed_id = 0;
		{
			let feed = ChainlinkFeed::feed(feed_id).expect("feed should be there");
			assert_eq!(feed.first_valid_round(), None);
			assert_eq!(feed.latest_round(), 0);
			assert_eq!(feed.latest_data(), RoundDataOf::<Test>::default());
		}
		let round_id = 1;
		let submission = 42;
		assert_ok!(ChainlinkFeed::submit(
			Origin::signed(oracle),
			feed_id,
			round_id,
			submission
		));
		assert_ok!(ChainlinkFeed::submit(
			Origin::signed(second_oracle),
			feed_id,
			round_id,
			submission
		));
		{
			let mut feed = ChainlinkFeed::feed(feed_id).expect("feed should be there");
			assert_eq!(feed.first_valid_round(), Some(1));
			assert_eq!(feed.latest_round(), 1);
			assert_eq!(
				feed.latest_data(),
				RoundData {
					answer: 42,
					started_at: 0,
					updated_at: 0,
					answered_in_round: 1,
				}
			);

			assert_ok!(feed.request_new_round(AccountId::default()));
		}
		let round_id = 2;
		let round =
			ChainlinkFeed::round(feed_id, round_id).expect("second round should be present");
		assert_eq!(
			round,
			Round {
				started_at: 0,
				..Default::default()
			}
		);
	});
}

#[test]
fn payment_withdrawal_should_work() {
	new_test_ext().execute_with(|| {
		let amount = ExistentialDeposit::get();
		let oracle = 3;
		let admin = 4;
		let recipient = 5;
		Oracles::<Test>::insert(
			oracle,
			OracleMeta {
				withdrawable: amount,
				admin,
				..Default::default()
			},
		);
		assert_noop!(
			ChainlinkFeed::withdraw_payment(Origin::signed(admin), 123, recipient, amount),
			Error::<Test>::OracleNotFound
		);
		assert_noop!(
			ChainlinkFeed::withdraw_payment(Origin::signed(123), oracle, recipient, amount),
			Error::<Test>::NotAdmin
		);
		assert_noop!(
			ChainlinkFeed::withdraw_payment(Origin::signed(admin), oracle, recipient, 2 * amount),
			Error::<Test>::InsufficientFunds
		);
		let fund = FeedModuleId::get().into_account();
		let fund_balance = Balances::free_balance(&fund);
		Balances::make_free_balance_be(&fund, ExistentialDeposit::get());
		assert!(
			ChainlinkFeed::withdraw_payment(Origin::signed(admin), oracle, recipient, amount).is_err()
		);
		Balances::make_free_balance_be(&fund, fund_balance);

		assert_ok!(ChainlinkFeed::withdraw_payment(
			Origin::signed(admin),
			oracle,
			recipient,
			amount
		));
	});
}

#[test]
fn funds_withdrawal_should_work() {
	new_test_ext().execute_with(|| {
		let amount = 50;
		let recipient = 5;
		let fund = FeedModuleId::get().into_account();
		assert_noop!(
			ChainlinkFeed::withdraw_funds(Origin::signed(123), recipient, amount),
			Error::<Test>::NotPalletAdmin
		);
		assert_noop!(
			ChainlinkFeed::withdraw_funds(Origin::signed(fund), recipient, 101 * MIN_RESERVE),
			Error::<Test>::InsufficientFunds
		);
		assert_noop!(
			ChainlinkFeed::withdraw_funds(Origin::signed(fund), recipient, 100 * MIN_RESERVE),
			Error::<Test>::InsufficientReserve
		);
		assert_ok!(ChainlinkFeed::withdraw_funds(
			Origin::signed(fund),
			recipient,
			amount
		));
	});
}

#[test]
fn transfer_pallet_admin_should_work() {
	new_test_ext().execute_with(|| {
		let new_admin = 23;
		let fund = FeedModuleId::get().into_account();
		assert_noop!(ChainlinkFeed::transfer_pallet_admin(
			Origin::signed(123),
			new_admin
		), Error::<Test>::NotPalletAdmin);
		assert_ok!(ChainlinkFeed::transfer_pallet_admin(
			Origin::signed(fund),
			new_admin
		));
		assert_eq!(PendingPalletAdmin::<Test>::get(), Some(new_admin));
		assert_noop!(ChainlinkFeed::accept_pallet_admin(
			Origin::signed(123)
		), Error::<Test>::NotPendingPalletAdmin);
		assert_ok!(ChainlinkFeed::accept_pallet_admin(Origin::signed(
			new_admin
		)));
		assert_eq!(PalletAdmin::<Test>::get(), new_admin);
		assert_eq!(PendingPalletAdmin::<Test>::get(), None);
	});
}

#[test]
fn prune_should_work() {
	// ## Pruning Testing Scenario
	//
	// |- round zero
	// v             v- latest round
	// 0 1 2 3 4 5 6 7 8 <- reporting round
	//       ^- first valid round
	new_test_ext().execute_with(|| {
		let feed_id = 0;
		let oracle_a = 2;
		let oracle_b = 3;
		let oracle_admin = 4;
		let submission = 42;
		let submit_a = |r| {
			assert_ok!(ChainlinkFeed::submit(
				Origin::signed(oracle_a),
				feed_id,
				r,
				submission
			));
		};
		let submit_a_and_b = |r| {
			submit_a(r);
			assert_ok!(ChainlinkFeed::submit(
				Origin::signed(oracle_b),
				feed_id,
				r,
				submission
			));
		};

		let owner = 1;
		// we require min 2 oracles so that we can time out the first few
		// so first_valid_round will be > 1
		assert_ok!(FeedBuilder::new()
			.owner(owner)
			.timeout(1)
			.min_submissions(2)
			.restart_delay(0)
			.oracles(vec![(oracle_a, oracle_admin), (oracle_b, oracle_admin)])
			.build_and_store());

		System::set_block_number(1);
		// submit 2 rounds that will be timed out
		submit_a(1);
		System::set_block_number(3);
		submit_a(2);
		System::set_block_number(5);
		assert_noop!(
			ChainlinkFeed::prune(Origin::signed(owner), feed_id, 1, 3),
			Error::<Test>::NoValidRoundYet
		);
		// submit the valid rounds
		submit_a_and_b(3);
		assert_noop!(
			ChainlinkFeed::prune(Origin::signed(owner), feed_id, 1, 3),
			Error::<Test>::NothingToPrune
		);
		submit_a_and_b(4);
		submit_a_and_b(5);
		submit_a_and_b(6);
		submit_a_and_b(7);
		// simulate an unfinished round so reporting_round != latest_round
		submit_a(8);

		let first_to_prune = 1;
		let keep_round = 5;
		// failure cases
		assert_noop!(
			ChainlinkFeed::prune(Origin::signed(owner), feed_id, 0, keep_round),
			Error::<Test>::CannotPruneRoundZero
		);
		assert_noop!(
			ChainlinkFeed::prune(Origin::signed(owner), feed_id, 6, keep_round),
			Error::<Test>::NothingToPrune
		);
		assert_noop!(
			ChainlinkFeed::prune(Origin::signed(owner), 23, first_to_prune, keep_round),
			Error::<Test>::FeedNotFound
		);
		assert_noop!(
			ChainlinkFeed::prune(Origin::signed(23), feed_id, first_to_prune, keep_round),
			Error::<Test>::NotFeedOwner
		);
		assert_noop!(
			ChainlinkFeed::prune(Origin::signed(owner), feed_id, 4, keep_round),
			Error::<Test>::PruneContiguously
		);

		// do the successful prune
		assert_ok!(ChainlinkFeed::prune(
			Origin::signed(owner),
			feed_id,
			first_to_prune,
			keep_round
		));
		// we try to prune until 5, but limits are set up in a way that we can
		// only prune until 4
		assert_eq!(ChainlinkFeed::round(feed_id, 3), None);
		let round = ChainlinkFeed::round(feed_id, 4).expect("fourth round should be present");
		assert_eq!(
			round,
			Round {
				started_at: 5,
				answer: Some(submission),
				updated_at: Some(5),
				answered_in_round: Some(4),
			}
		);
	});
}

#[test]
fn feed_creation_permissioning() {
	new_test_ext().execute_with(|| {
		let admin = FeedModuleId::get().into_account();
		let new_creator = 15;
		assert_noop!(
			FeedBuilder::new().owner(new_creator).build_and_store(),
			Error::<Test>::NotFeedCreator
		);
		assert_noop!(
			ChainlinkFeed::set_feed_creator(
				Origin::signed(123),
				new_creator
			),
			Error::<Test>::NotPalletAdmin
		);
		assert_ok!(ChainlinkFeed::set_feed_creator(
			Origin::signed(admin),
			new_creator
		));
		assert_ok!(FeedBuilder::new().owner(new_creator).build_and_store());
		assert_noop!(
			ChainlinkFeed::remove_feed_creator(
				Origin::signed(123),
				new_creator
			),
			Error::<Test>::NotPalletAdmin
		);
		assert_ok!(ChainlinkFeed::remove_feed_creator(
			Origin::signed(admin),
			new_creator
		));
		assert_noop!(
			FeedBuilder::new().owner(new_creator).build_and_store(),
			Error::<Test>::NotFeedCreator
		);
	});
}

#[test]
fn can_go_into_debt_and_repay() {
	new_test_ext().execute_with(|| {
		let admin: AccountId = FeedModuleId::get().into_account();
		let owner = 1;
		let oracle = 2;
		let payment = 33;
		assert_ok!(FeedBuilder::new()
			.payment(payment)
			.owner(owner)
			.oracles(vec![(oracle, 3), (3, 3)])
			.build_and_store());
		assert_eq!(ChainlinkFeed::debt(), 0);
		// ensure the fund is out of tokens
		Balances::make_free_balance_be(&admin, ExistentialDeposit::get());
		assert_ok!(ChainlinkFeed::submit(Origin::signed(oracle), 0, 1, 42));
		assert_eq!(ChainlinkFeed::debt(), payment);
		let new_funds = 2 * payment;
		Balances::make_free_balance_be(&admin, new_funds);
		// should be possible to reduce debt partially
		assert_ok!(ChainlinkFeed::reduce_debt(Origin::signed(admin), 10));
		assert_eq!(Balances::free_balance(admin), new_funds - 10);
		assert_eq!(ChainlinkFeed::debt(), payment - 10);
		// should be possible to overshoot in passing the amount correcting debt...
		assert_ok!(ChainlinkFeed::reduce_debt(Origin::signed(42), payment));
		// ... but will only correct the debt
		assert_eq!(Balances::free_balance(admin), new_funds - payment);
		assert_eq!(ChainlinkFeed::debt(), 0);
	});
}

#[test]
fn feed_life_cylce() {
	new_test_ext().execute_with(|| {
		let id = 0;
		let owner = 1;
		let payment = 33;
		let timeout = 10;
		let submission_value_bounds = (1, 1_000);
		let submission_count_bounds = (1, 3);
		let decimals = 5;
		let description = b"desc".to_vec();
		let restart_delay = 1;
		let new_config = FeedConfig {
			owner,
			pending_owner: None,
			payment,
			timeout,
			submission_value_bounds,
			submission_count_bounds,
			decimals,
			description,
			restart_delay,
			latest_round: Zero::zero(),
			reporting_round: Zero::zero(),
			first_valid_round: None,
			oracle_count: Zero::zero(),
		};
		let oracles = vec![(2, 2), (3, 3), (4, 4)];
		{
			let mut feed = Feed::<Test>::new(id, new_config.clone());
			assert_ok!(feed.add_oracles(oracles.clone()));
			assert_ok!(feed.update_future_rounds(payment, submission_count_bounds, restart_delay, timeout));
		}
		let new_config = FeedConfig {
			oracle_count: oracles.len() as u32,
			..new_config.clone()
		};
		// config should be stored on drop
		assert_eq!(ChainlinkFeed::feed_config(id), Some(new_config.clone()));
		let new_timeout = 5;
		{
			let mut feed = Feed::<Test>::load_from(id).expect("feed should be there");
			feed.config.timeout = new_timeout;
		}
		let modified_config = FeedConfig {
			timeout: new_timeout,
			..new_config.clone()
		};
		// modified config should be stored on drop
		assert_eq!(ChainlinkFeed::feed_config(id), Some(modified_config.clone()));
		let ignored_timeout = 23;
		{
			let mut feed = Feed::<Test>::read_only_from(id).expect("feed should be there");
			feed.config.timeout = ignored_timeout;
		}
		// read only access should not store changes
		assert_eq!(ChainlinkFeed::feed_config(id), Some(modified_config.clone()));
		{
			let mut feed = ChainlinkFeed::feed_mut(id).expect("feed should be there");
			assert_ok!(feed.request_new_round(AccountId::default()));
		}
		assert_eq!(ChainlinkFeed::feed_config(id).unwrap().reporting_round, 1);
	});
}
