// DOT Mog, Susbstrate Gamification Project with C# .NET Standard & Unity3D
// Copyright (C) 2020-2021 DOT Mog Team, darkfriend77 & metastar77
//
// DOT Mog is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License.
// DOT Mog is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

use frame_support::{codec::{Encode, Decode}};
use sp_std::vec::{Vec};

#[derive(Encode, Decode, Clone, PartialEq)]
pub enum GameConfigType {
	Activated = 0,
	MaxMogwaisInAccount = 1,
	MaxStashSize = 2,
	AccountNaming = 3,
}

impl Default for GameConfigType { fn default() -> Self { Self::Activated } }

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct GameConfig{
	pub parameters: Vec<u8>
}

impl GameConfig {
	
	const PARAM_COUNT: u8 = 8;

	pub fn new() -> Self {
		let mut v = Vec::new();
		for i in 0..GameConfig::PARAM_COUNT {
			v.push(i);
		}
		return GameConfig {
			parameters: v,
		};
	}
}