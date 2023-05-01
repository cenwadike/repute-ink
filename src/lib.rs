//! this is of minimum implemntation of repute basic design
//!
//! This contract implement a time-activity based reputation management system

#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod repute_demo {
    use ink::prelude::string::String;
    use ink::storage::Mapping;

    pub type GlobalEpoch = u64;
    pub type GlobalEpochEra = u64;
    pub type GlobalEpochMultiplier = u64;
    pub type ReputationScore = u128;
    pub type UserReputationEpoch = u64;
    pub type Rank = String;

    /// Global contract state
    ///
    /// keep track of system and user specific data
    #[ink(storage)]
    pub struct ReputeDemo {
        /// record current reputation epoch
        pub epoch: GlobalEpoch,
        /// map reputation era to baseline multiplier
        pub epoch_reputation_multiplier: Mapping<GlobalEpochEra, GlobalEpochMultiplier>,
        /// map user account is to reputation score
        pub user_identifiers: Mapping<AccountId, (ReputationScore, UserReputationEpoch, Rank)>,
    }

    /// Event emitted when a user registration occurs.
    #[ink(event)]
    pub struct UserRegistered {
        #[ink(topic)]
        user: AccountId,
        epoch: UserReputationEpoch,
    }

    /// Event emitted when time based reputation is generated.
    #[ink(event)]
    pub struct ScoreGenerated {
        #[ink(topic)]
        user_epoch: UserReputationEpoch,
        epoch_era: GlobalEpochEra,
    }

    /// Event emitted when a request to update epoch era is made.
    #[ink(event)]
    pub struct EraAndMultiplier {
        #[ink(topic)]
        new_epoch: GlobalEpochEra,
        #[ink(topic)]
        new_multiplier: GlobalEpochMultiplier,
    }

    impl ReputeDemo {
        #[ink(constructor)]
        pub fn new(multiplier: GlobalEpochMultiplier) -> Self {
            let mut instance = Self::default();
            instance.epoch = Self::env().block_number().into();
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
        /// return false if user is already registered.
        #[ink(message)]
        pub fn register(&mut self) -> bool {
            let user = self.env().caller();

            if self.user_identifiers.contains(&user) {
                false
            } else {
                // create a new user data
                let reputation_score: u128 = 0;
                let user_epoch = self.epoch;
                let rank: String = String::from("_");

                // add new user data to state
                self.user_identifiers
                    .insert(user, &(reputation_score, user_epoch, rank));

                // emit registration event
                Self::env().emit_event(UserRegistered {
                    user,
                    epoch: user_epoch,
                });

                true
            }
        }

        /// get reputation
        /// only registered user can call this method
        ///
        /// mocks user on-chain activity to update user reputation
        #[ink(message)]
        pub fn get_reputation(&mut self) -> u128 {
            let user = self.env().caller();
            let mut user_identifier = self
                .user_identifiers
                .get(&user)
                .expect("Not a registered user");
            let score;

            // calculate new reputation score if user epoch is out of sync with global epoch era
            if user_identifier.1 == self.epoch {
                score = user_identifier.0;
            } else {
                // get user reputation score
                // @notice: call to reputation score generator
                let raw_score = self.calculate_reputation_score(user_identifier.1, self.epoch);

                // calculate user generated reputation using epoch multiplier
                score = raw_score
                    .checked_mul(
                        self.epoch_reputation_multiplier
                            .get(self.epoch)
                            .unwrap()
                            .into(),
                    )
                    .unwrap();

                // update user epoch
                user_identifier.1 = self.epoch;
            }

            // side effect to check if era is over and update to next era and era multiplier
            self.update_era();

            score
        }

        /// get a registered user reputation
        /// any user can call this method
        #[ink(message)]
        pub fn get_user_reputation(&self, user_id: AccountId) -> (ReputationScore, Rank) {
            let (score, _, rank) = self
                .user_identifiers
                .get(&user_id)
                .expect("Not a registered user");

            (score, rank)
        }

        /// calculate user reputation score
        ///
        /// mock call to an external time-based reputation engine
        /// calculate user reputation to latest epoch
        ///
        /// side effect of call is to trigger reputation for next epoch
        fn calculate_reputation_score(
            &self,
            user_epoch: UserReputationEpoch,
            epoch_era: GlobalEpochEra,
        ) -> u128 {
            // this is a simple time based reputation engine
            // returns the ratio of user epoch and a reference era
            let score = user_epoch
                .checked_mul(100)
                .unwrap()
                .checked_div(epoch_era)
                .unwrap();

            // emit generator event
            Self::env().emit_event(ScoreGenerated {
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
                .expect("No multiplier for this epoch");

            // check if 24 hours have passed (ie. ~14400 new blocks)
            if (Self::env().block_number() as u64)
                .checked_sub(current_era)
                .unwrap()
                > 14400
            {
                // update epoch era
                self.epoch = Self::env().block_number().into();
                let new_epoch = self.epoch;

                // update multiplier
                // increase multiplier by 1%
                let new_multiplier = current_multiplier
                    .checked_add(current_multiplier.checked_div(10).unwrap())
                    .unwrap();
                self.epoch_reputation_multiplier
                    .insert(new_epoch, &new_multiplier);

                // emit epoch current era and multiplier as event
                Self::env().emit_event(EraAndMultiplier {
                    new_epoch,
                    new_multiplier,
                });
            } else {
                return;
            }
        }
    }
}
