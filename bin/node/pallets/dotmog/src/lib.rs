#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure, codec::{Encode, Decode}, dispatch, traits::{Get, Randomness, Currency, ReservableCurrency, ExistenceRequirement::AllowDeath}};
use frame_system::{ensure_signed};
use sp_runtime::{traits::{Hash, TrailingZeroInput, Zero}};
use sp_std::vec::Vec;
use rand_chacha::{rand_core::{RngCore, SeedableRng}, ChaChaRng};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const MAX_AUCTIONS_PER_BLOCK: usize = 2;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MogwaiStruct<Hash, BlockNumber, Balance> {
	id: Hash,
	dna: Hash,
	genesis: BlockNumber,
	price: Balance,
	gen: u64,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Auction<Hash, Balance, BlockNumber, AccountId> {
	mogwai_id: Hash,
	mogwai_owner: AccountId,
	expiry: BlockNumber,
	min_bid: Balance,
	high_bid: Balance,
	high_bidder: AccountId,
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Currency: ReservableCurrency<Self::AccountId>;
	type Randomness: Randomness<Self::Hash>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as DotMogModule {
		
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Something get(fn something): Option<u32>;

		/// A map of mogwais accessible by the mogwai hash.
		Mogwais get(fn mogwai): map hasher(identity) T::Hash => MogwaiStruct<T::Hash, T::BlockNumber, BalanceOf<T>>;
		/// A map of mogwai owners accessible by the mogwai hash.
		MogwaiOwner get(fn owner_of): map hasher(identity) T::Hash => Option<T::AccountId>;
				
		/// A map of all existing mogwais accessible by the index. 
		AllMogwaisArray get(fn mogwai_by_index): map hasher(blake2_128_concat) u64 => T::Hash;
		/// A count over all existing mogwais in the system.
		AllMogwaisCount get(fn all_mogwais_count): u64;
		/// A map of the index of the mogwai accessible by the mogwai hash.
		AllMogwaisIndex: map hasher(identity) T::Hash => u64;
		
		/// A map of all mogwai hashes associated with an account.
		OwnedMogwaisArray get(fn mogwai_of_owner_by_index): map hasher(blake2_128_concat) (T::AccountId, u64) => T::Hash;
		/// A count over all existing mogwais owned by one account.
		OwnedMogwaisCount get(fn owned_mogwais_count): map hasher(blake2_128_concat) T::AccountId => u64;
		/// A map of the owned mogwais index accessible by the mogwai hash.
		OwnedMogwaisIndex: map hasher(identity) T::Hash => u64;
		
		/// A map of mogwai auctions accessible by the mogwai hash.
		MogwaiAuction get(fn auction_of): map hasher(blake2_128_concat) T::Hash => Option<Auction<T::Hash, BalanceOf<T>, T::BlockNumber, T::AccountId>>;
		/// A vec of mogwai auctions accessible by the expiry block number. 
		Auctions get(fn auctions_expire_at): map hasher(blake2_128_concat) T::BlockNumber => Vec<Auction<T::Hash, BalanceOf<T>, T::BlockNumber, T::AccountId>>;
		/// Current auction period max limit.       
		AuctionPeriodLimit get(fn auction_period_limit): T::BlockNumber = 1000.into();
		
		/// A map of bids accessible by account id and mogwai hash.
		Bids get(fn bid_of): map hasher(blake2_128_concat) (T::Hash, T::AccountId) => BalanceOf<T>;
		
		/// A vec of accounts accessible by mogwai hash.
		BidAccounts get(fn bid_accounts): map hasher(blake2_128_concat) T::Hash => Vec<T::AccountId>;
		
		/// The nonce used for randomness.
		Nonce: u64 = 0;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> 
	where 
	AccountId = <T as frame_system::Trait>::AccountId,	
	Hash = <T as frame_system::Trait>::Hash,
	BlockNumber = <T as frame_system::Trait>::BlockNumber,
	Balance = BalanceOf<T> {

		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, AccountId),

		/// A mogwai has been created.
		MogwaiCreated(AccountId, Hash),

		/// A price has been set for a mogwai.
		PriceSet(AccountId, Hash, Balance),

		/// A mogwai changed his owner.
		Transferred(AccountId, AccountId, Hash),

		/// A mogwai has been was bought.
		Bought(AccountId, AccountId, Hash, Balance),

		/// A auction has been created
		AuctionCreated(Hash, Balance, BlockNumber),

		/// A bid has been placed.
		Bid(Hash, Balance, AccountId),

		/// A auction hash been finalized.
		AuctionFinalized(Hash, Balance, BlockNumber),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Error names should be descriptive.
		NoneValue,
		/// A Storage overflow, has occured make sure to validate first.
		StorageOverflow,
		/// The mogwai hash already exists.
		MogwaiAlreadyExists,
		/// The mogwais hash doesn't exist.
		MogwaiDoesntExists,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			Something::put(something);

			// Emit an event.
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			// Return a successful DispatchResult
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match Something::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					Something::put(new);
					Ok(())
				},
			}
		}

		/// Set price of mogwai.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
        fn set_price(origin, mogwai_id: T::Hash, new_price: BalanceOf<T>) -> dispatch::DispatchResult {

            let sender = ensure_signed(origin)?;

            ensure!(Mogwais::<T>::contains_key(mogwai_id), Error::<T>::MogwaiDoesntExists);

			let owner = Self::owner_of(mogwai_id).ok_or("No owner for this mogwai")?;
			
			ensure!(owner == sender, "You don't own this mogwai");

            let mut mogwai = Self::mogwai(mogwai_id);
            mogwai.price = new_price;

            <Mogwais<T>>::insert(mogwai_id, mogwai);

            Self::deposit_event(RawEvent::PriceSet(sender, mogwai_id, new_price));
            
            Ok(())
        }

		/// Create a new mogwai.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn create_mogwai(origin) -> dispatch::DispatchResult {
			
			let sender = ensure_signed(origin)?;

			//let test: &[u8] = $[u8,1];
			//let sparkle_heart = vec![0, 159, 146, 150];
			
			//let data_hash = T::Hashing::hash(random_bytes.as_bytes());

			let block_number = <frame_system::Module<T>>::block_number();
			//let block_hash = <frame_system::Module<T>>::block_hash(block_number);
			
			let random_hash = Self::generate_random_hash(b"create_mogwai", sender.clone());

			let new_mogwai = MogwaiStruct {
							id: random_hash,
							dna: random_hash,
							genesis: block_number,
							price: Zero::zero(),
							gen: 0,
			};

			Self::mint(sender, random_hash, new_mogwai)?;
			
			Ok(())
		}
		
		/// Transfer mogwai to a new account.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn transfer(origin, to: T::AccountId, mogwai_id: T::Hash) -> dispatch::DispatchResult {

            let sender = ensure_signed(origin)?;

			let owner = Self::owner_of(mogwai_id).ok_or("No owner for this mogwai")?;
			
			ensure!(owner == sender, "You don't own this mogwai");

            Self::transfer_from(sender, to, mogwai_id)?;

            Ok(())
		}
		
		/// Buy a mogwai.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn buy_mogwai(origin, mogwai_id: T::Hash, max_price: BalanceOf<T>) -> dispatch::DispatchResult {
			
			let sender = ensure_signed(origin)?;
			
			ensure!(Mogwais::<T>::contains_key(mogwai_id), Error::<T>::MogwaiDoesntExists);

			let owner = Self::owner_of(mogwai_id).ok_or("No owner for this mogwai")?;
			
			ensure!(owner != sender, "You already own this mogwai");
		
			let mut mogwai = Self::mogwai(mogwai_id);
		
			let mogwai_price = mogwai.price;

			ensure!(!mogwai_price.is_zero(), "You can't buy this mogwai, there is no price");
		
			ensure!(mogwai_price <= max_price, "You can't buy this mogwai, price exceeds your max price limit");

			T::Currency::transfer(&sender, &owner, mogwai_price, AllowDeath)?;

			// Transfer the mogwai using `transfer_from()` including a proof of why it cannot fail
			Self::transfer_from(owner.clone(), sender.clone(), mogwai_id)
				.expect("`owner` is shown to own the mogwai; \
				`owner` must have greater than 0 mogwai, so transfer cannot cause underflow; \
				`all_mogwai_count` shares the same type as `owned_mogwai_count` \
				and minting ensure there won't ever be more than `max()` mogwais, \
				which means transfer cannot cause an overflow; \
				qed");
			
			// Reset mogwai price back to zero, and update the storage
			mogwai.price = Zero::zero();

			<Mogwais<T>>::insert(mogwai_id, mogwai);

			Self::deposit_event(RawEvent::Bought(sender, owner, mogwai_id, mogwai_price));
			
			Ok(())
		}

		/// Breed a mogwai.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn breed_mogwai(origin, mogwai_id_1: T::Hash, mogwai_id_2: T::Hash) -> dispatch::DispatchResult {
			
			let sender = ensure_signed(origin)?;

			ensure!(Mogwais::<T>::contains_key(mogwai_id_1), Error::<T>::MogwaiDoesntExists);
			ensure!(Mogwais::<T>::contains_key(mogwai_id_2), Error::<T>::MogwaiDoesntExists);

			let parents = [Self::mogwai(mogwai_id_1) , Self::mogwai(mogwai_id_2)];

			let next_gen = parents[0].gen + parents[1].gen + 1;

			let random_hash = Self::generate_random_hash(b"breed_mogwai", sender.clone());

			let mut final_dna = parents[0].dna;
			for (i, (dna_2_element, r)) in parents[1].dna.as_ref().iter().zip(random_hash.as_ref().iter()).enumerate() {
				if r % 2 == 0 {
					final_dna.as_mut()[i] = *dna_2_element;
				}
			}

			let block_number = <frame_system::Module<T>>::block_number();
			//let block_hash = <frame_system::Module<T>>::block_hash(block_number);

			let new_mogwai = MogwaiStruct {
                id: random_hash,
				dna: final_dna,
				genesis: block_number,
                price: Zero::zero(),
				gen: next_gen,
            };

			Self::mint(sender, random_hash, new_mogwai)?;

			Ok(())
		}

		/// Create a new auction.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn create_auction(origin, mogwai_id: T::Hash, min_bid: BalanceOf<T>, expiry: T::BlockNumber) -> dispatch::DispatchResult {
			
			let sender = ensure_signed(origin)?;

			ensure!(Mogwais::<T>::contains_key(mogwai_id), Error::<T>::MogwaiDoesntExists);

			let owner = Self::owner_of(mogwai_id).ok_or("No owner for this mogwai")?;

            ensure!(owner == sender, "You can't set an auction for a mogwai you don't own");

            ensure!(expiry > <frame_system::Module<T>>::block_number(), "The expiry has to be greater than the current block number");
            ensure!(expiry <= <frame_system::Module<T>>::block_number() + Self::auction_period_limit(), "The expiry has be lower than the limit block number");

            let auctions = Self::auctions_expire_at(expiry);
            ensure!(auctions.len() < MAX_AUCTIONS_PER_BLOCK, "Maximum number of auctions is reached for the target block, try another block");

            let new_auction = Auction {
                mogwai_id,
                mogwai_owner: owner,
                expiry,
                min_bid,
                high_bid: min_bid,
                high_bidder: sender,
            };

            <MogwaiAuction<T>>::insert(mogwai_id, &new_auction);
            <Auctions<T>>::mutate(expiry, |auctions| auctions.push(new_auction.clone()));

            Self::deposit_event(RawEvent::AuctionCreated(mogwai_id, min_bid, expiry));

            Ok (())
		}
		
		/// Bid on an auction.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn bid_auction(origin, mogwai_id: T::Hash, bid: BalanceOf<T>) -> dispatch::DispatchResult {
            let sender = ensure_signed(origin)?;

			ensure!(Mogwais::<T>::contains_key(mogwai_id), Error::<T>::MogwaiDoesntExists);

			let owner = Self::owner_of(mogwai_id).ok_or("No owner for this mogwai")?;

            ensure!(owner != sender, "You can't bid for your own mogwai");

			let mut auction = Self::auction_of(mogwai_id).ok_or("No auction for this mogwai")?;
			
            ensure!(<frame_system::Module<T>>::block_number() < auction.expiry, "This auction is expired.");

            ensure!(bid > auction.high_bid, "Your bid has to be greater than the highest bid.");

            ensure!(T::Currency::free_balance(&sender) >= bid, "You don't have enough free balance for this bid");

            auction.high_bid = bid;
            auction.high_bidder = sender.clone();

            <MogwaiAuction<T>>::insert(mogwai_id, &auction);
            <Auctions<T>>::mutate(auction.expiry, |auctions| {
                for stored_auction in auctions {
                    if stored_auction.mogwai_id == mogwai_id {
                        *stored_auction = auction.clone();
                    }
                }
            });

            if <Bids<T>>::contains_key((mogwai_id, sender.clone())) {
                let escrow_balance = Self::bid_of((mogwai_id, sender.clone()));
                T::Currency::reserve(&sender, bid - escrow_balance)?;
            } else {
                T::Currency::reserve(&sender, bid)?;
            }
            <Bids<T>>::insert((mogwai_id, sender.clone()), bid);
            <BidAccounts<T>>::mutate(mogwai_id, |accounts| accounts.push(sender.clone()));

            Self::deposit_event(RawEvent::Bid(mogwai_id, auction.high_bid, auction.high_bidder));

            Ok (())
		}
		
		/// On finalize
		fn on_finalize() {

			let auctions = Self::auctions_expire_at(<frame_system::Module<T>>::block_number());

			for auction in &auctions {

                let owned_mogwais_count_from = Self::owned_mogwais_count(&auction.mogwai_owner);

				let owned_mogwais_count_to = Self::owned_mogwais_count(&auction.high_bidder);

				if owned_mogwais_count_to.checked_add(1).is_some() &&
				   owned_mogwais_count_from.checked_sub(1).is_some() &&
                   auction.mogwai_owner != auction.high_bidder
                {
					<MogwaiAuction<T>>::remove(auction.mogwai_id);

					let _ = T::Currency::unreserve(&auction.high_bidder, auction.high_bid);
					
					let _currency_transfer = T::Currency::transfer(&auction.high_bidder, &auction.mogwai_owner, auction.high_bid, AllowDeath);

					match _currency_transfer {
                        Err(_e) => continue,
                        Ok(_v) => {
                            let _kitty_transfer = Self::transfer_from(auction.mogwai_owner.clone(), auction.high_bidder.clone(), auction.mogwai_id);
                            match _kitty_transfer {
                                Err(_e) => continue,
                                Ok(_v) => {
                                    Self::deposit_event(RawEvent::AuctionFinalized(auction.mogwai_id, auction.high_bid, auction.expiry));
                                },
                            }
                        },
                    }
                }
			}
			
            for auction in &auctions {

				<Auctions<T>>::remove(<frame_system::Module<T>>::block_number());
				
				let bid_accounts = Self::bid_accounts(auction.mogwai_id);

				for account in bid_accounts {

                    let bid_balance = Self::bid_of((auction.mogwai_id, account.clone()));
                    let _ = T::Currency::unreserve(&account, bid_balance);
                    <Bids<T>>::remove((auction.mogwai_id, account));
				}
				
				<BidAccounts<T>>::remove(auction.mogwai_id);
            }
		}
	}
}

impl<T: Trait> Module<T> {

	/// Reads the nonce from storage, increments the stored nonce, and returns
	/// the encoded nonce to the caller.
	fn encode_and_update_nonce() -> Vec<u8> {
		let nonce = Nonce::get();
		Nonce::put(nonce.wrapping_add(1));
		nonce.encode()
	}

	fn mint(to: T::AccountId, mogwai_id: T::Hash, new_mogwai: MogwaiStruct<T::Hash, T::BlockNumber, BalanceOf<T>>) -> dispatch::DispatchResult {

		//ensure!(<MogwaiOwner<T>>::contains_key(&mogwai_id), "Mogwai already exists!");
		ensure!(!MogwaiOwner::<T>::contains_key(&mogwai_id), Error::<T>::MogwaiAlreadyExists);

		let owned_mogwais_count = Self::owned_mogwais_count(&to);
		let new_owned_mogwais_count = owned_mogwais_count.checked_add(1)
			.ok_or("Overflow adding a new mogwai to account balance")?;

		let all_mogwais_count = Self::all_mogwais_count();
		let new_all_mogwais_count = all_mogwais_count.checked_add(1)
			.ok_or("Overflow adding a new mogwai to total supply")?;

		// Update maps.
		<Mogwais<T>>::insert(mogwai_id, new_mogwai);
		<MogwaiOwner<T>>::insert(mogwai_id, &to);
			
		<AllMogwaisArray<T>>::insert(all_mogwais_count, mogwai_id);
        AllMogwaisCount::put(new_all_mogwais_count);
        <AllMogwaisIndex<T>>::insert(mogwai_id, all_mogwais_count);
			
		<OwnedMogwaisArray<T>>::insert((to.clone(), owned_mogwais_count), mogwai_id);
        <OwnedMogwaisCount<T>>::insert(&to, new_owned_mogwais_count);
        <OwnedMogwaisIndex<T>>::insert(mogwai_id, owned_mogwais_count);

		// Emit an event.
		Self::deposit_event(RawEvent::MogwaiCreated(to, mogwai_id));

		Ok(())
	}

	fn transfer_from(from: T::AccountId, to: T::AccountId, mogwai_id: T::Hash) -> dispatch::DispatchResult {

		let owner = Self::owner_of(mogwai_id).ok_or("No owner for this mogwai")?;

        ensure!(owner == from, "You don't own this mogwai");

        ensure!(!<MogwaiAuction<T>>::contains_key(mogwai_id), "This mogwai has an open auction.");

        let owned_mogwai_count_from = Self::owned_mogwais_count(&from);
        let owned_mogwai_count_to = Self::owned_mogwais_count(&to);

		let new_owned_mogwai_count_from = owned_mogwai_count_from.checked_sub(1)
			.ok_or("Overflow removing a mogwai from account")?;
		let new_owned_mogwai_count_to = owned_mogwai_count_to.checked_add(1)
			.ok_or("Overflow adding a mogwai to account")?;

        // NOTE: This is the "swap and pop" algorithm we have added for you
        //       We use our storage items to help simplify the removal of elements from the OwnedMogwaisArray
        //       We switch the last element of OwnedMogwaisArray with the element we want to remove
        let mogwai_index = <OwnedMogwaisIndex<T>>::get(mogwai_id);
        if mogwai_index != new_owned_mogwai_count_from {
            let last_mogwai_id = <OwnedMogwaisArray<T>>::get((from.clone(), new_owned_mogwai_count_from));
            <OwnedMogwaisArray<T>>::insert((from.clone(), mogwai_index), last_mogwai_id);
            <OwnedMogwaisIndex<T>>::insert(last_mogwai_id, mogwai_index);
        }

		// Now we can remove this item by removing the last element
		<MogwaiOwner<T>>::insert(mogwai_id, &to);
		<OwnedMogwaisIndex<T>>::insert(mogwai_id, owned_mogwai_count_to);

		<OwnedMogwaisArray<T>>::remove((from.clone(), new_owned_mogwai_count_from));
		<OwnedMogwaisArray<T>>::insert((to.clone(), owned_mogwai_count_to), mogwai_id);
		
		// Update the OwnedMogwaisCount for `from` and `to`
		<OwnedMogwaisCount<T>>::insert(&from, new_owned_mogwai_count_from);
		<OwnedMogwaisCount<T>>::insert(&to, new_owned_mogwai_count_to);

		// Emit an event.
		Self::deposit_event(RawEvent::Transferred(from, to, mogwai_id));

        Ok(())
	}

	fn generate_random_hash(phrase: &[u8], sender: T::AccountId) -> T::Hash {
		let seed = T::Randomness::random(phrase);
		let seed = <[u8; 32]>::decode(&mut TrailingZeroInput::new(seed.as_ref()))
			 .expect("input is padded with zeroes; qed");
		return (seed, &sender, Self::encode_and_update_nonce()).using_encoded(T::Hashing::hash);
	}
}

