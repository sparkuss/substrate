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

//impl BreedType {
//    fn from_u32(value: u32) -> BreedType {
//        match value {
//            0 => BreedType::DomDom,
//            1 => BreedType::DomRez,
//			2 => BreedType::RezDom,
//			3 => BreedType::RezRez,
//            _ => panic!("Unknown value: {}", value),
//        }
//    }
//}

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

struct Binary { }

impl Binary {
    pub fn get_bit_at(input: u8, n: u8) -> bool {
        input & (1 << n) != 0
    }
 //   pub fn bit_twiddling(original: u8, bit: u8) {
 //       let mask = 1 << bit;
 //       println!(
 //           "Original: {:#010b}, Set: {:#010b}, Cleared: {:#010b}, Toggled: {:#010b}",
 //           original,
 //           original |  mask,
 //           original & !mask,
 //           original ^  mask
 //       );
 //   }
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
