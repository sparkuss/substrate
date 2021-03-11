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

#[derive(Encode, Decode, Clone, PartialEq)]
pub enum BreedType {
	DomDom = 0,
	DomRez = 1,
	RezDom = 2,
	RezRez = 3,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq)]
pub enum RarityType {
	Minor = 0,
	Normal = 1,
	Rare = 2,
	Epic = 3,
    Legendary = 4,
}

impl Default for RarityType { fn default() -> Self { Self::Minor }}

impl RarityType { 
    pub fn from_u32(value: u32) -> RarityType {
        match value {
            0 => RarityType::Minor,
            1 => RarityType::Normal,
			2 => RarityType::Rare,
			3 => RarityType::Epic,
            4 => RarityType::Legendary,
            _ => RarityType::Minor,
        }
    }
}

pub struct Breeding;

impl Breeding {

	pub fn pairing(breed_type: BreedType, gen1: [u8;16], gen2: [u8;16]) -> [u8;32] {

		let mut final_dna : [u8;32] = [0;32];      
			
		let (ll, rr) = final_dna.split_at_mut(16);
		let (l1, l2) = ll.split_at_mut(8);
		let (r1, r2) = rr.split_at_mut(8);

		match breed_type {
			BreedType::DomDom => {
				l1.copy_from_slice(&gen1[..8]);
				l2.copy_from_slice(&gen1[8..16]);
				r1.copy_from_slice(&gen2[..8]);
				r2.copy_from_slice(&gen2[8..16]);
			}
			,
			BreedType::DomRez => {
				l1.copy_from_slice(&gen1[..8]);
				l2.copy_from_slice(&gen1[8..16]);
				r1.copy_from_slice(&gen2[8..16]);
				r2.copy_from_slice(&gen2[..8]);
			},
			BreedType::RezDom => {
				l1.copy_from_slice(&gen1[8..16]);
				l2.copy_from_slice(&gen1[..8]);
				r1.copy_from_slice(&gen2[8..16]);
				r2.copy_from_slice(&gen2[..8]);
			},
			BreedType::RezRez => {					
				l1.copy_from_slice(&gen1[8..16]);
				l2.copy_from_slice(&gen1[..8]);
				r1.copy_from_slice(&gen2[..8]);
				r2.copy_from_slice(&gen2[8..16]);
			},
		}
		return final_dna;
	}

    pub fn segmenting(gen: [u8;32], blk: [u8;32]) -> ([u8;16],[u8;16]) {
        
		let a_sec = &gen[0 .. 16];
		let b_sec = &gen[16 .. 32];
		
		//let a_x = &gen[0 ..  8];
		let a_y = &gen[8 .. 16];
		let b_x = &gen[16 .. 24];
		//let b_y = &gen[24 .. 32];  
		
		let a_c = &a_y[0 .. 4];
		let b_c = &b_x[0 .. 4];
	
		let mut dna: [u8;16] = Default::default();
		let mut evo: [u8;16] = Default::default();

        let mut full: u8 = 0;
        let mut mark: u8 = 0;

        for i in 0..32 {
        
            let bit_a = Binary::get_bit_at(a_c[i / 8], i as u8 % 8);
            let bit_b = Binary::get_bit_at(b_c[i / 8], i as u8 % 8);
    
            let p1:usize = i*2;
            let p2:usize = i+1;
            let blk_a = Binary::get_bit_at(blk[p1/8], p1 as u8 % 8);
            let blk_b = Binary::get_bit_at(blk[p2/8], p2 as u8 % 8);
    
            let mut half_byte: u8 = dna[i/2];
            let mut mark_byte: u8 = evo[i/2];
    
            let a_byte = a_sec[i / 2];
            let b_byte = b_sec[i / 2];
            let side = i % 2;
    
            if side == 0 {
                full = 0;
                mark = 0;
            }
    
            // 1 - 0
            if bit_a && !bit_b {
                if blk_a {
                    half_byte = Binary::copy_bits(half_byte, a_byte, side); // A+ as 4
                    half_byte = Binary::add_one(half_byte, side);
                    mark_byte = Binary::copy_bits(mark_byte, 0x44, side);
    
                } else if !blk_b {
                    half_byte = Binary::copy_bits(half_byte, a_byte, side); // A as A
                    mark_byte = Binary::copy_bits(mark_byte, 0xAA, side);
                } else {
                    half_byte = Binary::copy_bits(half_byte, a_byte ^ b_byte, side); // A^B as 7
                    mark_byte = Binary::copy_bits(mark_byte, 0x77, side);
                }
            } else 
            // 0 - 1
            if !bit_a && bit_b {
                if blk_b {
                    half_byte = Binary::copy_bits(half_byte, b_byte, side); // 8
                    mark_byte = Binary::copy_bits(mark_byte, 0x88, side);
                    half_byte = Binary::add_one(half_byte, side);
                } else if !blk_a {
                    half_byte = Binary::copy_bits(half_byte, b_byte, side); // B
                    mark_byte = Binary::copy_bits(mark_byte, 0xBB, side);
                } else {
                    half_byte = Binary::copy_bits(half_byte, b_byte ^ a_byte, side); // A^B as 7
                    mark_byte = Binary::copy_bits(mark_byte, 0x77, side); 
                }  
            } else 
            // 0 - 0
            if !bit_a && !bit_b {
                if !blk_a && !blk_b  {
                    if bit_a < bit_b {
                        half_byte = Binary::copy_bits(half_byte, a_byte & !b_byte, side); // !b- as 1
                        half_byte = Binary::sub_one(half_byte, side);
                        mark_byte = Binary::copy_bits(mark_byte, 0x11, side);
                    } else {
                        half_byte = Binary::copy_bits(half_byte, !a_byte & b_byte, side); // !a- as 0
                        mark_byte = Binary::copy_bits(mark_byte, 0x00, side);
                        half_byte = Binary::sub_one(half_byte, side);
                    }
                } else if blk_a && blk_b {
                    half_byte = Binary::copy_bits(half_byte, !blk[i], side); // !blk as E
                    mark_byte = Binary::copy_bits(mark_byte, 0xEE, side);
                } else {
                    if blk_a {
                        half_byte = Binary::copy_bits(half_byte, a_byte, side); // A
                        mark_byte = Binary::copy_bits(mark_byte, 0xAA, side);
                    } else {
                        half_byte = Binary::copy_bits(half_byte, b_byte, side); // B
                        mark_byte = Binary::copy_bits(mark_byte, 0xBB, side);
                    }
                } 
            } else 
            // 1 - 1
            {           
                if blk_a && blk_b {
                    half_byte = Binary::copy_bits(half_byte, a_byte | b_byte, side); // |+ as C
                    half_byte = Binary::add_one(half_byte, side);
                    mark_byte = Binary::copy_bits(mark_byte, 0xCC, side);
                } else 
                if !blk_a && !blk_b {
                    half_byte = Binary::copy_bits(half_byte, blk[i], side); // blk as F
                    mark_byte = Binary::copy_bits(mark_byte, 0xFF, side);
                } else {
                    if blk_a {
                        half_byte = Binary::copy_bits(half_byte, a_byte, side); // A
                        mark_byte = Binary::copy_bits(mark_byte, 0xAA, side);
                    } else {
                        half_byte = Binary::copy_bits(half_byte, b_byte, side); // B
                        mark_byte = Binary::copy_bits(mark_byte, 0xBB, side);
                    }
                } 
            }
    
            full = Binary::copy_bits(full, half_byte, side);
            mark = Binary::copy_bits(mark, mark_byte, side);
    
            // recombination
            if side == 1 {
                if full == 0xFF || full == 0x00 {
                    full &= blk[i];
                    mark = 0x33;
                }
                dna[i/2] = full;
                evo[i/2] = mark;
            }
        }

        (dna,evo)
    }
}

pub struct Generation { }

impl Generation {

    pub fn next_gen(gen1: u32, rar1: RarityType, gen2: u32, rar2: RarityType, random_hash: &[u8]) -> (RarityType,u32) {
        
        let mut result: u32 = 1;
        
        let mut rarity1:u32 = 0;
        let mut rarity2:u32 = 0;

        let mut rar11 = rar1 as u32;
        let mut rar22 = rar2 as u32;

        if rar11 > 0 {
            rar11 -= 1;
        }

        if rar22 > 0 {
            rar22 -= 1;
        }

        let base_rar = (rar11 + rar22) / 2;

        if gen1 > 0 && gen1 < 17 && gen2 > 0 && gen2 < 17 && random_hash.len() == 32 {
            
            let rng_gen11 = random_hash[1] as u32 + random_hash[2] as u32;
            let rng_gen12 = random_hash[3] as u32 + random_hash[4] as u32;
            let rng_gen13 = random_hash[5] as u32 + random_hash[6] as u32;

            let rng_gen21 = random_hash[7] as u32 + random_hash[8] as u32;
            let rng_gen22 = random_hash[9] as u32 + random_hash[10] as u32;
            let rng_gen23 = random_hash[11] as u32 + random_hash[12] as u32;

            let mut gen1pow2 = gen1 * 2;
            if gen1pow2 >=  (rar11 * 2)
            {
                gen1pow2 -= rar11; 
            }

            let mut gen2pow2 = gen2 * 2; 
            if gen2pow2 >=  (rar22 * 2)
            {
                gen2pow2 -= rar22; 
            }

            let mut base_gen1 = gen1.clone();
            if (rng_gen11 % gen1pow2) == 0 {
                base_gen1 += 1;
                rarity1 = 1;
                if (rng_gen12 % gen1pow2) < (base_gen1 / 2) {
                    base_gen1 += 1;
                    rarity1 = 2;
                    if (rng_gen13 % gen1pow2) < (base_gen1 / 2) {
                        base_gen1 += 1;
                        rarity1 = 3;
                    } 
                } 
            } 
            else if (rng_gen13 % gen1pow2) == 0 {
                base_gen1 -= 1;
            }

            let mut base_gen2 = gen2.clone();
            if (rng_gen21 % gen2pow2) == 0 {
                base_gen2 += 1;
                rarity2 = 1;
                if (rng_gen22 % gen2pow2) < (base_gen2 / 2) {
                    base_gen2 += 1;
                    rarity2 = 2;
                    if (rng_gen23 % gen2pow2) < (base_gen1 / 2) {
                        base_gen2 += 1;
                        rarity2 = 3;
                    } 
                }
            } 
            else if (rng_gen23 % gen2pow2) == 0 {
                base_gen2 -= 1;
            }
            
            result = (base_gen1 + base_gen2 + base_rar) / 2;

            if result > 16 {
                result = 16;
            }
            else if result < 1 {
                result = 1;
            }
        }

        let rarity = RarityType::from_u32(((rarity1 + rarity2 + ((rar1 as u32 + rar2 as u32) / 2)) / 2) % 5);

        (rarity, result)
    }
}

struct Binary { }

impl Binary {

    pub fn get_bit_at(input: u8, n: u8) -> bool {
        input & (1 << n) != 0
    }

    pub fn copy_bits(mut old: u8, mut new: u8, side: usize) -> u8 {
        if side == 0 {
            new = new & 0xF0;
        } else {
            new = new & 0x0F;
        }
        old |= new;
        old
    }

    pub fn add_one(mut old: u8, side: usize) -> u8{
        let mut new = old.clone();
        if side == 0 {
            old = old & 0x0F;
            new >>= 4;
            new += 1;
            new <<= 4;
            if new == 0 {
                new = 0xF0;
            }
        } else {
            old = old & 0xF0;
            new = new & 0x0F;
            new += 1;
            new = new & 0x0F;
            if new == 0 {
                new = 0x0F;
            }
        }
        new |= old;
        new
    }

    pub fn sub_one(mut old: u8, side: usize) -> u8{
        let mut new = old.clone();
        if side == 0 {
            old = old & 0x0F;
            new >>= 4;
            if new != 0 {
                new -= 1;
            }
            new <<= 4;
            } else {
            old = old & 0xF0;
            new = new & 0x0F;
            if new > 0 {
                new -= 1;
            }
            new = new & 0x0F;
        }
        new |= old;
        new
    }
}
