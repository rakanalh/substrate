// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

use sc_service::config::SignerConfig;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use crate::arg_enums::SignerType;
use crate::error::Result;

/// Parameters of the signer
#[derive(Debug, StructOpt, Clone)]
pub struct SignerParams {
	/// Specify type of signer.
	#[structopt(long = "signer-type", value_name = "TYPE", default_value = "local" )]
	pub signer_type: SignerType,

	/// Signer host, only applicaple for RemoteClient signer type.
	#[structopt(long = "signer-host" )]
	pub signer_host: Option<String>,

	/// Signer port.
	/// If signer is RemoteClient, this specifies the port to connect to.
	/// If signer is RemoteServer, this specifies the port to listen on for connections.
	#[structopt(long = "signer-port")]
	pub signer_port: Option<u32>,
}

impl SignerParams {
	/// Get the keystore configuration for the parameters
	pub fn signer_config(&self) -> Result<SignerConfig> {
		Ok(SignerConfig {
			signer_type: self.signer_type.into(),
			host: self.signer_host.clone(),
			port: self.signer_port,
		})
	}
}
