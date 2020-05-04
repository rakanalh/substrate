use std::sync::Arc;
use futures::executor::block_on;
use parking_lot::RwLock;
use tonic;

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

const LOG_TARGET: &'static str = "signer";
use log::{debug};
impl Signer for LocalSigner {
	fn sign_with(
		&self,
		id: KeyTypeId,
		key: &CryptoTypePublicPair,
		msg: &[u8],
	) -> Result<Vec<std::primitive::u8>, BareCryptoStoreError> {
		debug!(
			target: LOG_TARGET,
			"SIGNING MESSAGE",
		);
		self.keystore.read().sign_with(id, key, msg)
	}

	fn supported_keys(
		&self,
		id: KeyTypeId,
	) -> Result<Vec<CryptoTypePublicPair>, BareCryptoStoreError> {
		self.keystore.read().supported_keys(id, vec![])
	}

}

pub mod RemoteGRPCSigner {
	tonic::include_proto!("remotesigner");
}

#[derive(Default)]
pub struct RemoteSigner {
	host: String,
	port: u32
}

impl RemoteSigner {
	pub fn new(host: String, port: u32) -> RemoteSigner {
		RemoteSigner {
			host,
			port
		}
	}
}

impl Signer for RemoteSigner {
	fn sign_with(
		&self,
		id: sp_application_crypto::KeyTypeId,
		key: &sp_application_crypto::CryptoTypePublicPair,
		msg: &[u8],
	) -> Result<Vec<u8>, BareCryptoStoreError> {
		use RemoteGRPCSigner::{
			signer_client::SignerClient,
			SignRequest
		};
		block_on(async {
			let mut client = SignerClient::connect("http://127.0.0.1:50051").await
				.map_err(|_| BareCryptoStoreError::Unavailable)?;

			let request = tonic::Request::new(SignRequest {
				message: "Tonic".into(),
			});

			let response = client.sign(request).await
				.map_err(|_| BareCryptoStoreError::Unavailable)?;
			Ok::<Vec<u8>, BareCryptoStoreError>(response.into_inner().message)
		})
	}

	fn supported_keys(
		&self,
		id: KeyTypeId,
	) -> Result<Vec<CryptoTypePublicPair>, BareCryptoStoreError> { todo!() }
}
