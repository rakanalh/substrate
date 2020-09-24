// This file is part of Substrate.

// Copyright (C) 2019-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Types that should only be used for testing!

use crate::crypto::KeyTypeId;

/// Key type for generic Ed25519 key.
pub const ED25519: KeyTypeId = KeyTypeId(*b"ed25");
/// Key type for generic Sr 25519 key.
pub const SR25519: KeyTypeId = KeyTypeId(*b"sr25");
/// Key type for generic Sr 25519 key.
pub const ECDSA: KeyTypeId = KeyTypeId(*b"ecds");

/// A task executor that can be used in tests.
///
/// Internally this just wraps a `ThreadPool` with a pool size of `8`. This
/// should ensure that we have enough threads in tests for spawning blocking futures.
#[cfg(feature = "std")]
#[derive(Clone)]
pub struct TaskExecutor(futures::executor::ThreadPool);

#[cfg(feature = "std")]
impl TaskExecutor {
	/// Create a new instance of `Self`.
	pub fn new() -> Self {
		let mut builder = futures::executor::ThreadPoolBuilder::new();
		Self(builder.pool_size(8).create().expect("Failed to create thread pool"))
	}
}

#[cfg(feature = "std")]
impl crate::traits::SpawnNamed for TaskExecutor {
	fn spawn_blocking(&self, _: &'static str, future: futures::future::BoxFuture<'static, ()>) {
		self.0.spawn_ok(future);
	}
	fn spawn(&self, _: &'static str, future: futures::future::BoxFuture<'static, ()>) {
		self.0.spawn_ok(future);
	}
}
