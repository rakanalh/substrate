// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Module helpers for off-chain calls.

use codec::Encode;
use sp_std::convert::TryFrom;
use sp_std::prelude::Vec;
use sp_runtime::app_crypto::{AppPublic, AppSignature, RuntimeAppPublic};
use sp_runtime::traits::{Extrinsic as ExtrinsicT, IdentifyAccount, One};
use frame_support::{debug, storage::StorageMap};

/// Marker enum used to flag using all supported keys to sign a payload.
pub enum ForAll {}
/// Marker enum used to flag using any of the supported keys to sign a payload.
pub enum ForAny {}

/// Provides the ability to directly submit signed and unsigned
/// transaction onchain.
///
/// For submitting unsigned transactions, `submit_unsigned_transaction`
/// utility function can be used. However, this struct is used by `Signer`
/// to submit a signed transactions providing the signature along with the call.
pub struct SubmitTransaction<T: SendTransactionTypes<OverarchingCall>, OverarchingCall> {
	_phantom: sp_std::marker::PhantomData<(T, OverarchingCall)>
}

impl<T, LocalCall> SubmitTransaction<T, LocalCall>
where
	T: SendTransactionTypes<LocalCall>,
{
	pub fn submit_transaction(
		call: <T as SendTransactionTypes<LocalCall>>::OverarchingCall,
		signature: Option<<T::Extrinsic as ExtrinsicT>::SignaturePayload>
	) -> Result<(), ()> {
		let xt = T::Extrinsic::new(call.into(), signature).ok_or(())?;
		sp_io::offchain::submit_transaction(xt.encode())
	}

	pub fn submit_unsigned_transaction(
		call: <T as SendTransactionTypes<LocalCall>>::OverarchingCall,
	) -> Result<(), ()> {
		SubmitTransaction::<T, LocalCall>::submit_transaction(call, None)
	}
}

/// Provides an implementation for signing transaction payloads
///
/// Keys used for signing are defined when instantiating the signer object.
/// Signing can be done using:
///
/// - All supported keys in the keystore
/// - Any of the supported keys in the keystore
/// - A list of provided keys
///
/// The signer is then able to:
/// - Submit a unsigned transaction with a signed payload
/// - Submit a signed transaction
pub struct Signer<T: SigningTypes, X = ForAny> {
	accounts: Option<Vec<T::Public>>,
	_phantom: sp_std::marker::PhantomData<X>,
}

impl<T: SigningTypes, X> Default for Signer<T, X> {
	fn default() -> Self {
		Self {
			accounts: Default::default(),
			_phantom: Default::default(),
		}
	}
}

impl<T: SigningTypes, X> Signer<T, X> {
	pub fn all_accounts() -> Signer<T, ForAll> {
		Default::default()
	}

	pub fn any_account() -> Signer<T, ForAny> {
		Default::default()
	}

	pub fn with_filter(mut self, accounts: Vec<T::Public>) -> Self {
		self.accounts = Some(accounts);
		self
	}
}


impl<T: SigningTypes> Signer<T, ForAll> {
	fn for_all<F, R>(&self, f: F) -> Vec<(Account<T>, R)> where
		F: Fn(&Account<T>) -> Option<R>,
	{
		if let Some(ref accounts) = self.accounts {
			accounts
				.iter()
				.enumerate()
				.filter_map(|(index, key)| {
					let account_id = key.clone().into_account();
					let account = Account::new(index, account_id, key.clone());
					f(&account).map(|res| (account, res))
				})
				.collect()
		} else {
			<T::Public as RuntimeAppPublic>::all()
				.into_iter()
				.enumerate()
				.filter_map(|(index, key)| {
					let generic_public = <T::Public as AppPublic>::Generic::from(key);
					let public = generic_public.into();
					let account_id = public.clone().into_account();
					let account = Account::new(index, account_id, public.clone());
					f(&account).map(|res| (account, res))
				})
				.collect()
		}
	}
}

impl<T: SigningTypes> Signer<T, ForAny> {
	fn for_any<F, R>(&self, f: F) -> Option<(Account<T>, R)> where
		F: Fn(&Account<T>) -> Option<R>,
	{
		if let Some(ref accounts) = self.accounts {
			for (index, key) in accounts.iter().enumerate() {
				let account_id = key.clone().into_account();
				let account = Account::new(index, account_id, key.clone());
				let res = f(&account);
				if let Some(res) = res {
					return Some((account, res));
				}
			}
		} else {
			let runtime_keys = <T::Public as RuntimeAppPublic>::all()
				.into_iter()
				.enumerate();

			for (index, key) in runtime_keys {
				let generic_public = <T::Public as AppPublic>::Generic::from(key);
				let public = generic_public.into();
				let account_id = public.clone().into_account();
				let account = Account::new(index, account_id, public.clone());
				let res = f(&account);
				if let Some(res) = res {
					return Some((account, res));
				}
			}
		}
		None
	}
}

impl<T: SigningTypes> SignMessage<T> for Signer<T, ForAll> {
	type Result = Vec<(Account<T>, T::Signature)>;

	fn sign_message(&self, message: &[u8]) -> Self::Result {
		self.for_all(|account| {
			T::sign(&message, account.public.clone())
		})
	}

	fn sign<TPayload, F>(&self, f: F) -> Self::Result where
		F: Fn(&Account<T>) -> TPayload,
		TPayload: SignedPayload<T>,
	{
		self.for_all(|account| f(account).sign())
	}
}

impl<T: SigningTypes> SignMessage<T> for Signer<T, ForAny> {
	type Result = Option<(Account<T>, T::Signature)>;

	fn sign_message(&self, message: &[u8]) -> Self::Result {
		self.for_any(|account| {
			T::sign(&message, account.public.clone())
		})
	}

	fn sign<TPayload, F>(&self, f: F) -> Self::Result where
		F: Fn(&Account<T>) -> TPayload,
		TPayload: SignedPayload<T>,
	{
		self.for_any(|account| f(account).sign())
	}
}

impl<
	T: CreateSignedTransaction<LocalCall> + SigningTypes,
	LocalCall,
> SendSignedTransaction<T, LocalCall> for Signer<T, ForAny> {
	type Result = Option<(Account<T>, Result<(), ()>)>;

	fn send_signed_transaction(
		&self,
		f: impl Fn(&Account<T>) -> LocalCall,
	) -> Self::Result {
			self.for_any(|account| {
			let call = f(account);
			self.submit_signed_transaction(account, call)
		})
	}
}

impl<
	T: SigningTypes + CreateSignedTransaction<LocalCall>,
	LocalCall,
> SendSignedTransaction<T, LocalCall> for Signer<T, ForAll> {
	type Result = Vec<(Account<T>, Result<(), ()>)>;

	fn send_signed_transaction(
		&self,
		f: impl Fn(&Account<T>) -> LocalCall,
	) -> Self::Result {
		self.for_all(|account| {
			let call = f(account);
			self.submit_signed_transaction(account, call)
		})
	}
}

impl<
	T: SigningTypes + SendTransactionTypes<LocalCall>,
	LocalCall,
> SendUnsignedTransaction<T, LocalCall> for Signer<T, ForAny> {
	type Result = Option<(Account<T>, Result<(), ()>)>;

	fn send_unsigned_transaction<TPayload, F>(
		&self,
		f: F,
		f2: impl Fn(TPayload, T::Signature) -> LocalCall,
	) -> Self::Result
	where
		F: Fn(&Account<T>) -> TPayload,
		TPayload: SignedPayload<T>
	{
		self.for_any(|account| {
			let payload = f(account);
			let signature= payload.sign()?;
			let call = f2(payload, signature);
			self.submit_unsigned_transaction(call)
		})
	}
}

impl<
	T: SigningTypes + SendTransactionTypes<LocalCall>,
	LocalCall,
> SendUnsignedTransaction<T, LocalCall> for Signer<T, ForAll> {
	type Result = Vec<(Account<T>, Result<(), ()>)>;

	fn send_unsigned_transaction<TPayload, F>(
		&self,
		f: F,
		f2: impl Fn(TPayload, T::Signature) -> LocalCall,
	) -> Self::Result
	where
		F: Fn(&Account<T>) -> TPayload,
		TPayload: SignedPayload<T> {
		self.for_all(|account| {
			let payload = f(account);
			let signature = payload.sign()?;
			let call = f2(payload, signature);
			self.submit_unsigned_transaction(call)
		})
	}
}

/// Account information used for signing payloads
pub struct Account<T: SigningTypes> {
	pub index: usize,
	pub id: T::AccountId,
	pub public: T::Public,
}

impl<T: SigningTypes> Account<T> {
	pub fn new(index: usize, id: T::AccountId, public: T::Public) -> Self {
		Self { index, id, public }
	}
}

impl<T: SigningTypes> Clone for Account<T> where
	T::AccountId: Clone,
	T::Public: Clone,
{
	fn clone(&self) -> Self {
		Self {
			index: self.index,
			id: self.id.clone(),
			public: self.public.clone(),
		}
	}
}

/// App specific crypto trait that provides sign/verify
/// abilities to offchain workers. Implementations of this
/// trait should specify the app-specific public/signature
/// types.
pub trait AppCrypto<Public, Signature> {
	type RuntimeAppPublic: RuntimeAppPublic;
	// TODO [ToDr] The conversions are messy, clean them up.
	//
	// The idea would be to have some implementation for `RuntimeAppPublic`
	// to convert to and from generic types.
	// Maybe even a method like:
	// impl RuntimeAppPublic {
	//  fn into_public<T: From<Self::Generic>>(&self) -> T;
	// }
	// so an ability to convert the runtime app public into
	// some type that is reachable from the inner (wrapped) generic
	// crypto type.
	// So example:
	// ImOnline(sr25519) = RuntimeAppPublic
	// sr25519 = Generic
	// MutliSigner = From<sr25519>
	type GenericPublic:
		From<Self::RuntimeAppPublic>
		+ Into<Self::RuntimeAppPublic>
		+ TryFrom<Public>
		+ Into<Public>;
	type GenericSignature:
		From<<Self::RuntimeAppPublic as RuntimeAppPublic>::Signature>
		+ Into<<Self::RuntimeAppPublic as RuntimeAppPublic>::Signature>
		+ TryFrom<Signature>
		+ Into<Signature>;


}

/// A wrapper around the types which are used for signing transactions.
/// This trait should be implemented on the runtime.
pub trait SigningTypes: crate::Trait {
	//type AccountId;
	// TODO [ToDr] Could this be just `T::Signature as traits::Verify>::Signer`?
	// Seems that this may cause issues with bounds resolution.
	type Public: Clone
		+ PartialEq
		+ IdentifyAccount<AccountId = Self::AccountId>
		+ core::fmt::Debug
		+ codec::Codec
		+ RuntimeAppPublic
		+ AppPublic;
	type Signature: Clone
		+ PartialEq
		+ core::fmt::Debug
		+ codec::Codec
		+ AppSignature
		+ From<<Self::Public as RuntimeAppPublic>::Signature>
		+ Into<<Self::Public as RuntimeAppPublic>::Signature>;

	fn sign(payload: &[u8], public: Self::Public) -> Option<Self::Signature> {
		<Self::Public as RuntimeAppPublic>::sign(&public, &payload)
			.map(|x| {
				let sig: Self::Signature = x.into();
				sig
			})
	}

	fn verify(payload: &[u8], public: Self::Public, signature: Self::Signature) -> bool {
		let signature = signature.into();

		<Self::Public as RuntimeAppPublic>::verify(&public, &payload, &signature)
	}
}

/// A wrapper around the transaction and call types
pub trait SendTransactionTypes<LocalCall> {
	type Extrinsic: ExtrinsicT<Call=Self::OverarchingCall> + codec::Encode;
	type OverarchingCall: From<LocalCall>;
}

/// Create signed transaction
///
/// Should be implemented by the runtime to sign transaction data
pub trait CreateSignedTransaction<LocalCall>: SendTransactionTypes<LocalCall> + SigningTypes {
	/// Attempt to create signed extrinsic data that encodes call from given account.
	///
	/// Runtime implementation is free to construct the payload to sign and the signature
	/// in any way it wants.
	/// Returns `None` if signed extrinsic could not be created (either because signing failed
	/// or because of any other runtime-specific reason).
	fn create_transaction(
		call: Self::OverarchingCall,
		public: Self::Public,
		account: Self::AccountId,
		nonce: Self::Index,
	) -> Option<(Self::OverarchingCall, <Self::Extrinsic as ExtrinsicT>::SignaturePayload)>;
}

/// Sign message payload
pub trait SignMessage<T: SigningTypes> {
	type Result;

	fn sign_message(&self, message: &[u8]) -> Self::Result;

	fn sign<TPayload, F>(&self, f: F) -> Self::Result where
		F: Fn(&Account<T>) -> TPayload,
		TPayload: SignedPayload<T>,
		;
}

/// Submit a signed transaction onchain
pub trait SendSignedTransaction<
	T: SigningTypes + CreateSignedTransaction<LocalCall>,
	LocalCall
> {
	type Result;

	fn send_signed_transaction(
		&self,
		f: impl Fn(&Account<T>) -> LocalCall,
	) -> Self::Result;

	fn submit_signed_transaction(
		&self,
		account: &Account<T>,
		call: LocalCall
	) -> Option<Result<(), ()>> {
		let mut account_data = crate::Account::<T>::get(&account.id);
		debug::native::debug!(
			target: "offchain",
			"Creating signed transaction from account: {:?} (nonce: {:?})",
			account.id,
			account_data.nonce,
		);
		let (call, signature) = T::create_transaction(
			call.into(),
			account.public.clone(),
			account.id.clone(),
			account_data.nonce
		)?;
		let res = SubmitTransaction::<T, LocalCall>
			::submit_transaction(call, Some(signature));

		if res.is_ok() {
			// increment the nonce. This is fine, since the code should always
			// be running in off-chain context, so we NEVER persists data.
			account_data.nonce += One::one();
			crate::Account::<T>::insert(&account.id, account_data);
		}

		Some(res)
	}
}

/// Submit an unsigned transaction onchain with a signed payload
pub trait SendUnsignedTransaction<
	T: SigningTypes + SendTransactionTypes<LocalCall>,
	LocalCall,
> {
	type Result;

	fn send_unsigned_transaction<TPayload, F>(
		&self,
		f: F,
		f2: impl Fn(TPayload, T::Signature) -> LocalCall,
	) -> Self::Result
	where
		F: Fn(&Account<T>) -> TPayload,
		TPayload: SignedPayload<T>;

	fn submit_unsigned_transaction(
		&self,
		call: LocalCall
	) -> Option<Result<(), ()>> {
		Some(SubmitTransaction::<T, LocalCall>
			::submit_unsigned_transaction(call.into()))
	}
}

/// Utility trait to be implemented on payloads
/// that should be signed and submitted onchain.
pub trait SignedPayload<T: SigningTypes>: Encode {
	fn public(&self) -> T::Public;

	fn sign(&self) -> Option<T::Signature> {
		self.using_encoded(|payload| {
			<T::Public as RuntimeAppPublic>::sign(&self.public(), &payload)
				.map(Into::into)
		})
	}

	fn verify(&self, signature: T::Signature) -> bool {
		self.using_encoded(|payload| {
			<T::Public as RuntimeAppPublic>::verify(&self.public(), &payload, &signature.into()).into()
		})
	}
}
