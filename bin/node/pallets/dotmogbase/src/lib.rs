#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_support::{
		ensure, codec::{Encode, Decode}, 
		traits::{
			Get, Currency, ReservableCurrency
		}};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{traits::{Zero}};
	use sp_std::vec::{Vec};

	#[derive(Encode, Decode, Clone, PartialEq)]
	pub enum ClaimState {
		None = 0,
		Registred = 1,
		Verified = 2,
		Secured = 3,
		Processed = 4,
		Holded = 5,
		Failed = 6,
		Cancelled = 7,
	}
	
	impl Default for ClaimState { fn default() -> Self { Self::None } }

	impl ClaimState { 
		pub fn from_u32(value: u32) -> ClaimState {
			match value {
				0 => ClaimState::None,
				1 => ClaimState::Registred,
				2 => ClaimState::Verified,
				3 => ClaimState::Secured,
				4 => ClaimState::Processed,
				5 => ClaimState::Holded,
				6 => ClaimState::Failed,
				7 => ClaimState::Cancelled,
				_ => ClaimState::None,
			}
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Encode, Decode, Default, Clone, PartialEq)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct MogwaicoinAddress<AccountId, ClaimState, Balance> {
		address: Vec<u8>,
		account: AccountId,
		signature:  Vec<u8>,
		state: ClaimState,
		balance: Balance,
	}

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	/// The current authority set.
	#[pallet::storage]
	#[pallet::getter(fn key)]
	pub(super) type Key<T: Config> = StorageValue<_,T::AccountId, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub key: T::AccountId,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { key: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<Key<T>>::put(&self.key);
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn account_claim)]
	pub type AccountClaim<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(T::AccountId, Vec<u8>),
		MogwaicoinAddress<T::AccountId, ClaimState, BalanceOf<T>>,
		ValueQuery
	>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),

		/// Key for claim registrations changed
		/// parameters. [who] 
		KeyChanged(T::AccountId),
	}
	
	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Account is already claimed.
		AccountClaimAlreadyExists,
		/// Account doesn't exists.
		AccountClaimDoesntExists,
		/// Incorrect signature size.
		SignatureSize,
		/// Incorrect address size.
		AddressSize,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn change_key(origin: OriginFor<T>, to: T::AccountId) -> DispatchResultWithPostInfo {

			let sender = ensure_signed(origin)?;

			ensure!(sender == Self::key(), "only the dot mog founder can change key.");
		
			<Key<T>>::put(&to);

			// Emit an event.
			Self::deposit_event(Event::KeyChanged(to));

			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn claim(origin: OriginFor<T>,  address: Vec<u8>, account: T::AccountId, signature: Vec<u8>) -> DispatchResultWithPostInfo {

            let sender = ensure_signed(origin)?;

			ensure!(sender == Self::key(), "only the dot mog founder can add claims.");

			ensure!(!AccountClaim::<T>::contains_key((account.clone(), address.clone())), Error::<T>::AccountClaimAlreadyExists);
			//let owner = Self::owner_of(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			//let last_mogwai_id = <AccountClaim<T>>::get((account.clone(), address.clone()));

			ensure!(address.len() == 34, Error::<T>::AddressSize);
			ensure!(signature.len() <= 256, Error::<T>::SignatureSize);

            let mogwaicoin_address = MogwaicoinAddress {
                address: address.clone(),
				account: account.clone(),
                signature: signature,
                state: ClaimState::None,
                balance: Zero::zero(),
            };

			<AccountClaim<T>>::insert((account, address), mogwaicoin_address);

			// Return a successful DispatchResultWithPostInfo
			Ok(Pays::No.into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn update_claim(origin: OriginFor<T>, address: Vec<u8>, account: T::AccountId, state: u32, balance: BalanceOf<T>) -> DispatchResultWithPostInfo {

            let sender = ensure_signed(origin)?;

			ensure!(sender == Self::key(), "only the dot mog founder can add claims.");

			ensure!(AccountClaim::<T>::contains_key((account.clone(), address.clone())), Error::<T>::AccountClaimDoesntExists);

			let mut mogwaicoin_address = Self::account_claim((&account, &address));

			mogwaicoin_address.state = ClaimState::from_u32(state);
			mogwaicoin_address.balance = balance;

			<AccountClaim<T>>::insert((account, address), mogwaicoin_address);

			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads(1) + T::DbWeight::get().writes(1))]
		pub fn remove_claim(origin: OriginFor<T>, address: Vec<u8>, account: T::AccountId) -> DispatchResultWithPostInfo {

            let sender = ensure_signed(origin)?;

			ensure!(sender == Self::key(), "only the dot mog founder can remove claims.");

			ensure!(AccountClaim::<T>::contains_key((account.clone(), address.clone())), Error::<T>::AccountClaimDoesntExists);

			<AccountClaim<T>>::remove((account, address));

			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(0)]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(().into())
				},
			}
		}
	}
}
