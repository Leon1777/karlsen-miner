#![allow(clippy::unreadable_literal)]
use crate::HashKls;
use log::info;
use std::ops::BitXor;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use tiny_keccak::Hasher;

use lazy_static::lazy_static;

#[derive(Clone)]
pub struct PowB3Hash {
    pub hasher: blake3::Hasher,
}

#[derive(Clone)]
pub struct PowFishHash {
    // set the cache here not hasher
    //pub context: Context,
}

const FNV_PRIME: u32 = 0x01000193;
const FULL_DATASET_ITEM_PARENTS: u32 = 512;
const NUM_DATASET_ACCESSES: u32 = 32;
const LIGHT_CACHE_ROUNDS: i32 = 3;

const LIGHT_CACHE_NUM_ITEMS: u32 = 1179641;
const FULL_DATASET_NUM_ITEMS: u32 = 37748717;
const SEED: Hash256 = Hash256([
    0xeb, 0x01, 0x63, 0xae, 0xf2, 0xab, 0x1c, 0x5a, 0x66, 0x31, 0x0c, 0x1c, 0x14, 0xd6, 0x0f, 0x42, 0x55, 0xa9, 0xb3,
    0x9b, 0x0e, 0xdf, 0x26, 0x53, 0x98, 0x44, 0xf1, 0x17, 0xad, 0x67, 0x21, 0x19,
]);

const SIZE_U32: usize = std::mem::size_of::<u32>();
const SIZE_U64: usize = std::mem::size_of::<u64>();

pub trait HashData {
    fn new() -> Self;
    fn from_hash(hash: &HashKls) -> Self;
    fn as_bytes(&self) -> &[u8];
    fn as_bytes_mut(&mut self) -> &mut [u8];

    #[inline(always)]
    fn get_as_u32(&self, index: usize) -> u32 {
        u32::from_le_bytes(self.as_bytes()[index * SIZE_U32..index * SIZE_U32 + SIZE_U32].try_into().unwrap())
    }

    #[inline(always)]
    fn set_as_u32(&mut self, index: usize, value: u32) {
        self.as_bytes_mut()[index * SIZE_U32..index * SIZE_U32 + SIZE_U32].copy_from_slice(&value.to_le_bytes())
    }

    #[inline(always)]
    fn get_as_u64(&self, index: usize) -> u64 {
        u64::from_le_bytes(self.as_bytes()[index * SIZE_U64..index * SIZE_U64 + SIZE_U64].try_into().unwrap())
    }

    #[inline(always)]
    fn set_as_u64(&mut self, index: usize, value: u64) {
        self.as_bytes_mut()[index * SIZE_U64..index * SIZE_U64 + SIZE_U64].copy_from_slice(&value.to_le_bytes())
    }
}

#[derive(Debug)]
pub struct Hash256([u8; 32]);

impl HashData for Hash256 {
    #[inline(always)]
    fn new() -> Self {
        Self([0; 32])
    }

    #[inline(always)]
    fn from_hash(hash: &HashKls) -> Self {
        Self(hash.0)
    }

    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    #[inline(always)]
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Hash512([u8; 64]);

impl HashData for Hash512 {
    #[inline(always)]
    fn new() -> Self {
        Self([0; 64])
    }

    //Todo check if filled with 0
    #[inline(always)]
    fn from_hash(hash: &HashKls) -> Self {
        let mut result = Self::new();
        let (first_half, _) = result.0.split_at_mut(hash.0.len());
        first_half.copy_from_slice(&hash.0);
        result
    }

    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    #[inline(always)]
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl BitXor<&Hash512> for &Hash512 {
    type Output = Hash512;

    #[inline(always)]
    fn bitxor(self, rhs: &Hash512) -> Self::Output {
        let mut hash = Hash512::new();

        for i in 0..64 {
            hash.0[i] = self.0[i] ^ rhs.0[i]
        }

        hash
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Hash1024([u8; 128]);

impl HashData for Hash1024 {
    #[inline(always)]
    fn new() -> Self {
        Self([0; 128])
    }

    //Todo check if filled with 0
    #[inline(always)]
    fn from_hash(hash: &HashKls) -> Self {
        let mut result = Self::new();
        let (first_half, _) = result.0.split_at_mut(hash.0.len());
        first_half.copy_from_slice(&hash.0);
        result
    }

    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    #[inline(always)]
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl Hash1024 {
    #[inline(always)]
    fn from_512s(first: &Hash512, second: &Hash512) -> Self {
        let mut hash = Self::new();
        let (first_half, second_half) = hash.0.split_at_mut(first.0.len());
        first_half.copy_from_slice(&first.0);
        second_half.copy_from_slice(&second.0);

        hash
    }
}

#[derive(Clone)]
pub struct Context {
    //pub light_cache: Box<[Hash512]>,
    //pub full_dataset: Option<Box<[Hash1024]>>,
}

static FULL_DATASET: OnceLock<Box<[Hash1024]>> = OnceLock::new();

lazy_static! {
static ref LIGHT_CACHE: Box<[Hash512]> = {
    //vec![Hash512::new(); LIGHT_CACHE_NUM_ITEMS as usize].into_boxed_slice()

    //println!("light cache processing started");
    let mut light_cache = vec![Hash512::new(); LIGHT_CACHE_NUM_ITEMS as usize].into_boxed_slice();
    //println!("light_cache[10] : {:?}", light_cache[10]);
    //println!("light_cache[42] : {:?}", light_cache[42]);
    Context::build_light_cache(&mut light_cache);
    //println!("light_cache[10] : {:?}", light_cache[10]);
    //println!("light_cache[42] : {:?}", light_cache[42]);
    //println!("light cache processing done");

    light_cache
};
}

#[inline(always)]
fn get_dataset_item(index: usize) -> Hash1024 {
    let dataset = FULL_DATASET.get_or_init(|| {
        let mut full_dataset = vec![Hash1024::new(); FULL_DATASET_NUM_ITEMS as usize].into_boxed_slice();
        prebuild_dataset(&mut full_dataset, &LIGHT_CACHE, num_cpus::get_physical());
        full_dataset
    });
    dataset[index]
}

#[inline(always)]
fn prebuild_dataset(full_dataset: &mut Box<[Hash1024]>, light_cache: &[Hash512], num_threads: usize) {
    info!("prebuilding dataset using {} threads", num_threads);
    let start = std::time::Instant::now();

    let total_items = full_dataset.len();
    let progress = Arc::new(AtomicUsize::new(0));
    std::thread::scope(|scope| {
        let mut threads = Vec::with_capacity(num_threads);

        let batch_size = full_dataset.len() / num_threads;
        let chunks = full_dataset.chunks_mut(batch_size);

        for (index, chunk) in chunks.enumerate() {
            let chunk_start = index * batch_size;
            let progress = Arc::clone(&progress);

            let thread_handle = scope.spawn(move || {
                for (i, item) in chunk.iter_mut().enumerate() {
                    *item = PowFishHash::calculate_dataset_item_1024(light_cache, chunk_start + i);
                    let done = progress.fetch_add(1, Ordering::Relaxed) + 1;
                    if done % 4_000_000 == 0 {
                        let percent = done * 100 / total_items;
                        info!("prebuilding full dataset: {}% ({}/{})", percent, done, total_items);
                    }
                }
            });

            threads.push(thread_handle);
        }

        for handle in threads {
            handle.join().unwrap();
        }
    });

    info!("prebuilding full dataset done in {:.1}s", start.elapsed().as_secs_f64());
}

impl Context {
    #[inline(always)]
    fn build_light_cache(cache: &mut [Hash512]) {
        let mut item: Hash512 = Hash512::new();
        PowFishHash::keccak(&mut item.0, &SEED.0);
        cache[0] = item;

        for cache_item in cache.iter_mut().take(LIGHT_CACHE_NUM_ITEMS as usize).skip(1) {
            PowFishHash::keccak_in_place(&mut item.0);
            *cache_item = item;
        }

        for _ in 0..LIGHT_CACHE_ROUNDS {
            for i in 0..LIGHT_CACHE_NUM_ITEMS {
                // First index: 4 first bytes of the item as little-endian integer
                let t: u32 = cache[i as usize].get_as_u32(0);
                let v: u32 = t % LIGHT_CACHE_NUM_ITEMS;

                // Second index
                let w: u32 = (LIGHT_CACHE_NUM_ITEMS.wrapping_add(i.wrapping_sub(1))) % LIGHT_CACHE_NUM_ITEMS;

                let x = &cache[v as usize] ^ &cache[w as usize];
                PowFishHash::keccak(&mut cache[i as usize].0, &x.0);
            }
        }
    }
}

impl PowFishHash {
    #[inline(always)]
    pub fn fishhashplus_kernel(seed: &HashKls, use_dataset: bool) -> HashKls {
        let seed_hash512 = Hash512::from_hash(seed);
        let mut mix = Hash1024::from_512s(&seed_hash512, &seed_hash512);

        // FishhashPlus
        for i in 0..NUM_DATASET_ACCESSES {
            // Calculate new fetching indexes
            let mut mix_group: [u32; 8] = [0; 8];

            for (c, mix_group_elem) in mix_group.iter_mut().enumerate() {
                *mix_group_elem = mix.get_as_u32(4 * c)
                    ^ mix.get_as_u32(4 * c + 1)
                    ^ mix.get_as_u32(4 * c + 2)
                    ^ mix.get_as_u32(4 * c + 3);
            }

            let p0 = (mix_group[0] ^ mix_group[3] ^ mix_group[6]) % FULL_DATASET_NUM_ITEMS;
            let p1 = (mix_group[1] ^ mix_group[4] ^ mix_group[7]) % FULL_DATASET_NUM_ITEMS;
            let p2 = (mix_group[2] ^ mix_group[5] ^ i) % FULL_DATASET_NUM_ITEMS;

            // Use dataset lookup if available, otherwise light_cache (CPU)
            let fetch0 = if use_dataset {
                get_dataset_item(p0 as usize)
            } else {
                Self::calculate_dataset_item_1024(&LIGHT_CACHE, p0 as usize)
            };
            let mut fetch1 = if use_dataset {
                get_dataset_item(p1 as usize)
            } else {
                Self::calculate_dataset_item_1024(&LIGHT_CACHE, p1 as usize)
            };
            let mut fetch2 = if use_dataset {
                get_dataset_item(p2 as usize)
            } else {
                Self::calculate_dataset_item_1024(&LIGHT_CACHE, p2 as usize)
            };

            // Modify fetch1 and fetch2
            for j in 0..32 {
                fetch1.set_as_u32(j, PowFishHash::fnv1(mix.get_as_u32(j), fetch1.get_as_u32(j)));
                fetch2.set_as_u32(j, mix.get_as_u32(j) ^ fetch2.get_as_u32(j));
            }

            // Final computation of new mix
            for j in 0..16 {
                mix.set_as_u64(
                    j,
                    //fetch0.get_as_u64(j) * fetch1.get_as_u64(j) + fetch2.get_as_u64(j),
                    fetch0.get_as_u64(j).wrapping_mul(fetch1.get_as_u64(j)).wrapping_add(fetch2.get_as_u64(j)),
                );
            }
        }

        // Collapse the result into 32 bytes
        let mut mix_hash = Hash256::new();
        let num_words = std::mem::size_of_val(&mix) / SIZE_U32;

        for i in (0..num_words).step_by(4) {
            let h1 = PowFishHash::fnv1(mix.get_as_u32(i), mix.get_as_u32(i + 1));
            let h2 = PowFishHash::fnv1(h1, mix.get_as_u32(i + 2));
            let h3 = PowFishHash::fnv1(h2, mix.get_as_u32(i + 3));
            mix_hash.set_as_u32(i / 4, h3);
        }

        HashKls::from_bytes(mix_hash.0)
    }

    #[inline(always)]
    pub fn keccak(out: &mut [u8], data: &[u8]) {
        let mut hasher = tiny_keccak::Keccak::v512();
        hasher.update(data);
        hasher.finalize(out);
    }

    #[inline(always)]
    fn keccak_in_place(data: &mut [u8]) {
        //TODO remove tiny_keccak with asm keccak
        let mut hasher = tiny_keccak::Keccak::v512();
        hasher.update(data);
        hasher.finalize(data);
    }

    #[inline(always)]
    fn fnv1(u: u32, v: u32) -> u32 {
        u.wrapping_mul(FNV_PRIME) ^ v
    }

    #[inline(always)]
    fn fnv1_512(u: Hash512, v: Hash512) -> Hash512 {
        let mut r = Hash512::new();

        for i in 0..r.0.len() / SIZE_U32 {
            r.set_as_u32(i, PowFishHash::fnv1(u.get_as_u32(i), v.get_as_u32(i)));
        }

        r
    }

    #[inline(always)]
    fn calculate_dataset_item_1024(light_cache: &[Hash512], index: usize) -> Hash1024 {
        let seed0 = (index * 2) as u32;
        let seed1 = seed0 + 1;

        let mut mix0 = light_cache[(seed0 % LIGHT_CACHE_NUM_ITEMS) as usize];
        let mut mix1 = light_cache[(seed1 % LIGHT_CACHE_NUM_ITEMS) as usize];

        let mix0_seed = mix0.get_as_u32(0) ^ seed0;
        let mix1_seed = mix1.get_as_u32(0) ^ seed1;

        mix0.set_as_u32(0, mix0_seed);
        mix1.set_as_u32(0, mix1_seed);

        PowFishHash::keccak_in_place(&mut mix0.0);
        PowFishHash::keccak_in_place(&mut mix1.0);

        let num_words: u32 = (std::mem::size_of_val(&mix0) / SIZE_U32) as u32;
        for j in 0..FULL_DATASET_ITEM_PARENTS {
            let t0 = PowFishHash::fnv1(seed0 ^ j, mix0.get_as_u32((j % num_words) as usize));
            let t1 = PowFishHash::fnv1(seed1 ^ j, mix1.get_as_u32((j % num_words) as usize));
            mix0 = PowFishHash::fnv1_512(mix0, light_cache[(t0 % LIGHT_CACHE_NUM_ITEMS) as usize]);
            mix1 = PowFishHash::fnv1_512(mix1, light_cache[(t1 % LIGHT_CACHE_NUM_ITEMS) as usize]);
        }

        PowFishHash::keccak_in_place(&mut mix0.0);
        PowFishHash::keccak_in_place(&mut mix1.0);

        Hash1024::from_512s(&mix0, &mix1)
    }
}

impl PowB3Hash {
    #[inline(always)]
    pub fn new(pre_pow_hash: HashKls, timestamp: u64) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&pre_pow_hash.as_bytes());
        hasher.update(&timestamp.to_le_bytes());
        let array: [u8; 32] = [0; 32];
        hasher.update(&array);
        Self { hasher }
    }

    #[inline(always)]
    pub fn finalize_with_nonce(mut self, nonce: u64) -> HashKls {
        self.hasher.update(&nonce.to_le_bytes());
        let hash = self.hasher.finalize();
        HashKls(*hash.as_bytes())
    }

    pub fn hash(my_hash: HashKls) -> HashKls {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&my_hash.as_bytes());
        let hash = hasher.finalize();
        HashKls(*hash.as_bytes())
    }
}
