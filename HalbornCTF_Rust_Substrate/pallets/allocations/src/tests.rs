/*
 * This file is part of the Malborn Chain distributed at https://github.com/Malborn/chain
 * Copyright (C) 2020  Malborn International
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#![cfg(test)]

use super::*;
use crate::{self as pallet_allocations};
use frame_support::{assert_noop, assert_ok, ord_parameter_types, parameter_types};
use frame_system::EnsureSignedBy;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Config<T>, Storage, Event<T>},
        EmergencyShutdown: pallet_pause::{Module, Call, Storage, Event<T>},
        Allocations: pallet_allocations::{Module, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}
impl frame_system::Config for Test {
    type Origin = Origin;
    type Call = Call;
    type BlockWeights = ();
    type BlockLength = ();
    type SS58Prefix = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
}
parameter_types! {
    pub const ExistentialDeposit: u64 = 2;
    pub const MaxLocks: u32 = 50;
}
impl pallet_balances::Config for Test {
    type Balance = u64;
    type Event = ();
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = MaxLocks;
    type AccountStore = frame_system::Pallet<Test>;
    type WeightInfo = ();
}

ord_parameter_types! {
    pub const ShutdownAdmin: u64 = 21;
}
impl pallet_pause::Config for Test {
    type Event = ();
    type PauseOrigin = EnsureSignedBy<ShutdownAdmin, u64>;
    type WeightInfo = ();
}

parameter_types! {
    pub const Oracle: u64 = 0;
    pub const Hacker: u64 = 1;
    pub const Grantee: u64 = 2;
    pub const Receiver: u64 = 3;
    pub const CoinsLimit: u64 = 1_000_000;
    pub const Fee: Perbill = Perbill::from_percent(10);
}
impl WithAccountId<u64> for Receiver {
    fn account_id() -> u64 {
        Receiver::get()
    }
}
impl Config for Test {
    type Event = ();
    type Currency = pallet_balances::Pallet<Self>;
    type ProtocolFee = Fee;
    type ProtocolFeeReceiver = Receiver;
    type MaximumCoinsEverAllocated = CoinsLimit;
    type ExistentialDeposit = <Test as pallet_balances::Config>::ExistentialDeposit;
    type WeightInfo = ();
}
type Errors = Error<Test>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}

#[test]
fn non_oracle_can_not_trigger_allocation() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Allocations::allocate_coins(
                Origin::signed(Hacker::get()),
                Grantee::get(),
                50,
                Vec::new(),
            ),
            Errors::OracleAccessDenied
        );
    })
}

/*#[test]
fn oracle_triggers_allocation() {
    new_test_ext().execute_with(|| {
        Allocations::initialize_members(&[Oracle::get()]);
        assert_eq!(Allocations::is_oracle(Oracle::get()), true);

        assert_ok!(Allocations::allocate_coins(
            Origin::signed(Oracle::get()),
            Grantee::get(),
            50,
            Vec::new(),
        ));

    })
}*/

#[test]
fn allocate_the_right_amount_of_coins_to_everyone() {
    new_test_ext().execute_with(|| {
        Allocations::initialize_members(&[Oracle::get()]);

        assert_eq!(Allocations::allocated_coins(), 0);
        assert_ok!(Allocations::allocate_coins(
            Origin::signed(Oracle::get()),
            Grantee::get(),
            50,
            Vec::new(),
        ));

        assert_eq!(Balances::free_balance(Grantee::get()), 45);
        assert_eq!(Balances::free_balance(Receiver::get()), 5);
        assert_eq!(Allocations::allocated_coins(), 50);
    })
}


 /*#[test]
 fn can_not_allocate_more_coins_than_max() {
     new_test_ext().execute_with(|| {
         Allocations::initialize_members(&[Oracle::get()]);

         assert_noop!(
             Allocations::allocate_coins(
                 Origin::signed(Oracle::get()),
                 Grantee::get(),
                 CoinsLimit::get() + 1,
                 Vec::new(),
             ),
             Errors::TooManyCoinsToAllocate
         );
     })
}*/

#[test]
#[should_panic]
fn integer_overflow_allocating_coins() {
    new_test_ext().execute_with(|| {
        Allocations::initialize_members(&[Oracle::get()]);

        assert_noop!(
            Allocations::allocate_coins(
                Origin::signed(Oracle::get()),
                Grantee::get(),
                CoinsLimit::get() + 1,
                Vec::new(),
            ),
            Errors::TooManyCoinsToAllocate
        );

    })
}

#[test]
#[should_panic]
fn unchecked_vec_access() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            Allocations::allocate_coins(
                Origin::signed(Oracle::get()),
                Grantee::get(),
                50,
                vec![0; u32::MAX as usize + 1], // Exceeding u32::MAX
            )
        );
    })
}

/*#[test]
fn unused_proof_parameter() {
    new_test_ext().execute_with(|| {
        Allocations::initialize_members(&[Oracle::get()]);

        // Pass an empty proof vector, which is unused in the implementation.
        assert_ok!(Allocations::allocate_coins(
            Origin::signed(Oracle::get()),
            Grantee::get(),
            50,
            Vec::new(),
        ));
    })
}*/

#[test]
/*fn should_fail_allocate_coins_without_existential_deposit() {
    new_test_ext().execute_with(|| {
        // Set up test parameters
        let to = Hacker::get();
        let amount = 100u32.into();
        let proof = vec![1; 32]; // Example proof

        // Ensure the oracle is initialized
        Oracles::<Test>::put(vec![to.clone()]);

        // Ensure the allocation fails without sufficient existential deposit
        assert_noop!(
            Allocations::allocate_coins(
                Origin::signed(to.clone()),
                to.clone(),
                amount,
                proof.clone(),
            ),
            Error::<Test>::InsufficientExistentialDeposit
        );
    });
}*/

#[test]
fn invalid_fee_receiver_test() {
    new_test_ext().execute_with(|| {
        Allocations::initialize_members(&[Oracle::get()]);

        assert_eq!(Allocations::allocated_coins(), 0);
        assert_ok!(Allocations::allocate_coins(
            Origin::signed(Oracle::get()),
            Receiver::get(),
            50,
            Vec::new(),
        ));

        assert_eq!(Balances::free_balance(Receiver::get()), 50);
        assert_eq!(Allocations::allocated_coins(), 50);
    })
}


