use std::sync::Arc;
use parking_lot::RwLock;
use sp_core::traits::{BareCryptoStorePtr, BareCryptoStoreError, Signer};
use sp_core::{
	crypto::{CryptoTypePublicPair, KeyTypeId},
	traits::BareCryptoStore
};
use crate::Store;

pub struct LocalSigner {
	keystore: BareCryptoStorePtr,
}

impl LocalSigner {
	pub fn new(keystore: BareCryptoStorePtr) -> LocalSigner {
		LocalSigner {
			keystore,
		}
	}
}

impl Signer for LocalSigner {
	fn sign_with(
		&self,
		id: KeyTypeId,
		key: &CryptoTypePublicPair,
		msg: &[u8],
	) -> Result<Vec<std::primitive::u8>, BareCryptoStoreError> {
		self.keystore.read().sign_with(id, key, msg)
	}

	fn supported_keys(
		&self,
		id: KeyTypeId,
	) -> Result<Vec<CryptoTypePublicPair>, BareCryptoStoreError> {
		self.keystore.read().supported_keys(id, vec![])
	}

}

#[derive(Default)]
pub struct RemoteRestSigner {}

impl Signer for RemoteRestSigner {
	fn sign_with(
		&self,
		id: sp_application_crypto::KeyTypeId,
		key: &sp_application_crypto::CryptoTypePublicPair,
		msg: &[u8],
	) -> Result<Vec<u8>, BareCryptoStoreError> {
		Err(BareCryptoStoreError::Unavailable)
	}

	fn supported_keys(
		&self,
		id: KeyTypeId,
	) -> Result<Vec<CryptoTypePublicPair>, BareCryptoStoreError> { todo!() }
}
