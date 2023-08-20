//! this is of minimum implemntation of repute basic design
//!
//! This contract implement a time-activity based reputation management system
//!
//! `Types`:
//!
//! `Events`:
//!
//! `Constructor`:
//!
//! * new(multiplier: GlobalEpochMultiplier) -> Self
//!
//! `Change methods`:
//!
//!  * register(&mut self)
//!  * update_reputation(&mut self)
//!
//!  `View methods`:
//!
//!  * get_user_reputation(&self, user_id: AccountId) -> (ReputationScore, UserReputationEpoch, Rank)
//!
//!
//!  `Internal methods`:
//!
//!  * calculate_reputation_score(&self, user_epoch: UserReputationEpoch, epoch_era: GlobalEpochEra) -> ReputationScore
//!  * update_era(&mut self)
//!

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod repute {
    use ink::storage::Mapping;

    pub type GlobalEpoch = u32;
    pub type GlobalEpochEra = u32;
    pub type GlobalEpochMultiplier = u32;
    pub type RegisteredAt = u32;
    pub type ReputationScore = u128;
    pub type UserReputationEpoch = u32;

    #[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
    )]
    pub enum Rank {
        Rookie,
        SideKick,
        Hero,
    }

    /// Global contract state
    ///
    /// keep track of system and user specific data
    #[ink(storage)]
    pub struct Repute {
        /// record current reputation epoch
        pub epoch: GlobalEpoch,
        /// map reputation era to baseline multiplier
        pub epoch_reputation_multiplier: Mapping<GlobalEpochEra, GlobalEpochMultiplier>,
        /// map user account to reputation score
        pub user_identifiers:
            Mapping<AccountId, (RegisteredAt, ReputationScore, UserReputationEpoch, Rank)>,
    }

    /// Event emitted when a user registration occurs.
    #[ink(event)]
    pub struct UserRegistered {
        #[ink(topic)]
        user: AccountId,
        epoch: UserReputationEpoch,
    }

    /// Event emitted when time based reputation is calculated.
    #[ink(event)]
    pub struct ScoreCalculated {
        #[ink(topic)]
        user_epoch: UserReputationEpoch,
        epoch_era: GlobalEpochEra,
    }

    /// Event emitted when user reputation is generated.
    #[ink(event)]
    pub struct ScoreGenerated {
        #[ink(topic)]
        user: AccountId,
        #[ink(topic)]
        user_epoch: UserReputationEpoch,
        epoch_era: GlobalEpochEra,
    }

    /// Event emitted when a request to update epoch era is made.
    #[ink(event)]
    pub struct EraAndMultiplierUpdated {
        #[ink(topic)]
        new_epoch: GlobalEpochEra,
        #[ink(topic)]
        new_multiplier: GlobalEpochMultiplier,
    }

    impl Repute {
        #[ink(constructor)]
        pub fn new(multiplier: GlobalEpochMultiplier) -> Self {
            let mut instance = Self::default();

            instance.epoch = Self::env().block_number();
            instance
                .epoch_reputation_multiplier
                .insert(instance.epoch, &multiplier);
            instance.user_identifiers = Mapping::default();

            instance
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(GlobalEpochMultiplier::MIN)
        }

        /// register user
        ///
        /// caller's account id is used as identifier to register user on the platform
        ///
        /// panic if user is already registered.
        #[ink(message)]
        pub fn register(&mut self) {
            let user = self.env().caller();

            if self.user_identifiers.contains(&user) {
                panic!("ERR: User already registered");
            }

            // create a new user data
            let now = self.env().block_number();
            let reputation_score = ReputationScore::default();
            let user_epoch = self.epoch;
            let rank: Rank = Rank::Rookie;

            // add new user data to state
            self.user_identifiers
                .insert(user, &(now, reputation_score, user_epoch, rank));

            // emit event for indexing
            Self::env().emit_event(UserRegistered {
                user,
                epoch: user_epoch,
            });
        }

        /// update reputation
        /// only registered user can call this method
        ///
        /// mocks user on-chain activity to update user reputation
        #[ink(message)]
        pub fn update_reputation(&mut self) {
            let user = self.env().caller();
            let mut user_identifier = self
                .user_identifiers
                .get(&user)
                .expect("Err: Must be a registered user");

            // update reputation score if user epoch is out of sync with global epoch era
            if user_identifier.2 == self.epoch {
                return;
            }

            let existence = self
                .env()
                .block_number()
                .checked_sub(user_identifier.0 as u32)
                .unwrap();

            let six_months: u32 = 14400 * 30 * 6;
            let twelve_months: u32 = 14400 * 30 * 12;

            // >6 months to be a sidekick
            if existence > six_months {
                user_identifier.3 = Rank::SideKick;
            }

            // >12 months to be a hero
            if existence > twelve_months {
                user_identifier.3 = Rank::Hero
            }

            // get user reputation score
            // @notice: call to reputation score generator
            let raw_score = self.calculate_reputation_score(user_identifier.2, self.epoch);

            // calculate user generated reputation using epoch multiplier
            raw_score
                .checked_mul(
                    self.epoch_reputation_multiplier
                        .get(self.epoch)
                        .unwrap()
                        .into(),
                )
                .unwrap();

            // update user epoch
            user_identifier.2 = self.epoch;

            // save update to storage
            self.user_identifiers.insert(user, &user_identifier);

            // side effect to check if era is over and update to next era and era multiplier
            self.update_era();

            // emit event for indexing
            Self::env().emit_event(ScoreGenerated {
                user,
                user_epoch: user_identifier.2,
                epoch_era: self.epoch,
            });
        }

        /// get a registered user reputation
        /// any user can call this method
        #[ink(message)]
        pub fn get_user_reputation(
            &self,
            user_id: AccountId,
        ) -> (ReputationScore, UserReputationEpoch, Rank) {
            let (_, score, epoch, rank) = self
                .user_identifiers
                .get(&user_id)
                .expect("Err: Must be a registered user");

            (score, epoch, rank)
        }

        /// calculate user reputation score
        ///
        /// mock call to an external time-based reputation engine
        /// calculate user reputation in latest epoch
        fn calculate_reputation_score(
            &self,
            user_epoch: UserReputationEpoch,
            epoch_era: GlobalEpochEra,
        ) -> ReputationScore {
            // this is a simple time based reputation engine
            // returns the ratio of user epoch and a reference era
            let score = user_epoch
                .checked_mul(100)
                .unwrap()
                .checked_div(epoch_era)
                .unwrap();

            // emit event for indexing
            Self::env().emit_event(ScoreCalculated {
                user_epoch,
                epoch_era,
            });

            score.into()
        }

        /// update epoch and move to a new era
        /// update new epoch multiplier
        fn update_era(&mut self) {
            let current_era = self.epoch;
            let current_multiplier = self
                .epoch_reputation_multiplier
                .get(current_era)
                .expect("Err: Multiplier for current epoch");

            // check if 24 hours have passed (ie. ~14400 new blocks at 6sec/block)
            if (Self::env().block_number())
                .checked_sub(current_era)
                .unwrap()
                > 14400
            {
                // update epoch era
                self.epoch = Self::env().block_number();
                let new_epoch = self.epoch;

                // update multiplier
                // increase multiplier by 1%
                let new_multiplier = current_multiplier
                    .checked_add(current_multiplier.checked_div(100).unwrap())
                    .unwrap();

                self.epoch_reputation_multiplier
                    .insert(new_epoch, &new_multiplier);

                // emit event for indexing
                Self::env().emit_event(EraAndMultiplierUpdated {
                    new_epoch,
                    new_multiplier,
                });
            } else {
                return;
            }
        }
    }
}
