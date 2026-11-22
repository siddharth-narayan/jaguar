#![no_std]
#![feature(const_index)]
#![feature(const_iter)]
#![feature(const_trait_impl)]
#![feature(allocator_ext)]
#![feature(generic_atomic)]
#![feature(uint_gather_scatter_bits)]
extern crate alloc;

pub mod table;
pub mod brc;
pub mod json;