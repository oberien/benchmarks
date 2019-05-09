#![feature(test)]
#![feature(proc_macro_hygiene)]

extern crate test;

mod entities;
mod entities_phf;
mod entities_match;

#[cfg(test)]
mod tests {
    use std::iter;
    use std::ops::Deref;
    use std::collections::{HashMap, BTreeMap, hash_map::RandomState};
    use std::hash::{BuildHasher, BuildHasherDefault};

    use test::Bencher;

    use rand::{SeedableRng, seq::SliceRandom, distributions::{Distribution, Uniform, Alphanumeric}};
    use rand_xoshiro::Xoshiro256Plus;
    use eytzinger::{SliceExt, permutation::InplacePermutator};
    use seahash::SeaHasher;
    use fnv::FnvBuildHasher;

    use super::*;
    use crate::entities::ENTITIES;
    use crate::entities_phf::ENTITIES_PHF;

    fn samples() -> Vec<Vec<u8>> {
        let seed = [42u8; 32];
        let mut rng = Xoshiro256Plus::from_seed(seed);
        let mut samples = Vec::with_capacity(100);
        let len_uniform = Uniform::new(2, 6);

        // 90 samples from the array
        for _ in 0..90 {
            samples.push(ENTITIES.choose(&mut rng).unwrap().0.into());
        }

        // 10 samples not in the array
        for _ in 90..100 {
            loop {
                let len = len_uniform.sample(&mut rng);
                let bytes: Vec<_> = iter::repeat_with(|| Alphanumeric.sample(&mut rng))
                    .filter(|c| c.is_ascii_alphabetic())
                    .map(|c| c as u8)
                    .take(len)
                    .collect();
                if entities::get_entity(&bytes).is_some() {
                    continue
                }

                samples.push(bytes);
                break;
            }
        }

        samples.shuffle(&mut rng);

        samples
    }

    #[bench]
    fn via_match(b: &mut Bencher) {
        let samples = samples();
        let samples = &samples;
        b.iter(|| for sample in samples {
            test::black_box(entities_match::get_entity(&sample));
        });
    }

    #[bench]
    fn binary_search(b: &mut Bencher) {
        let samples = samples();
        let samples = &samples;
        b.iter(|| for sample in samples {
            let res = ENTITIES.binary_search_by_key(&sample.deref(), |&(key, _value)| key)
                .ok()
                .map(|idx| ENTITIES[idx].1);
            test::black_box(res);
        });
    }

    #[bench]
    fn eytzinger(b: &mut Bencher) {
        let mut samples = samples();
        eytzinger::eytzingerize(&mut samples, &mut InplacePermutator);
        let samples = &samples;
        b.iter(|| for sample in samples {
            let res = ENTITIES.eytzinger_search_by_key(&sample.deref(), |&(key, _value)| key)
                .map(|idx| ENTITIES[idx].1);
            test::black_box(res);
        });
    }

    // can't use a generic function because statics aren't monomorphized
    macro_rules! hashmap_lazy_static {
        ($hasher:ty, $b:ident) => {
            lazy_static::lazy_static! {
                static ref MAP: HashMap<&'static [u8], &'static str, $hasher> = {
                    make_hashmap::<$hasher>()
                };
            }

            hashmap_lookup(MAP.deref(), $b);
        }
    }

    fn make_hashmap<S: BuildHasher + Default>() -> HashMap<&'static [u8], &'static str, S> {
        let mut map = HashMap::with_capacity_and_hasher(ENTITIES.len(), S::default());
        map.extend(ENTITIES.iter().cloned());
        map
    }

    fn hashmap_lookup<S: BuildHasher>(map: &HashMap<&'static [u8], &'static str, S>, b: &mut Bencher) {
        let samples = samples();
        let samples = &samples;
        b.iter(|| for sample in samples {
            test::black_box(map.get(&sample.deref()));
        });
    }

    #[bench]
    fn hashmap_siphash(b: &mut Bencher) {
        let map = make_hashmap::<RandomState>();
        hashmap_lookup(&map, b);
    }

    #[bench]
    fn hashmap_seahash(b: &mut Bencher) {
        let map = make_hashmap::<BuildHasherDefault<SeaHasher>>();
        hashmap_lookup(&map, b);
    }

    #[bench]
    fn hashmap_fnv(b: &mut Bencher) {
        let map = make_hashmap::<FnvBuildHasher>();
        hashmap_lookup(&map, b);
    }

    #[bench]
    fn hashmap_siphash_lazy_static(b: &mut Bencher) {
        hashmap_lazy_static!(RandomState, b);
    }

    #[bench]
    fn hashmap_seahash_lazy_static(b: &mut Bencher) {
        hashmap_lazy_static!(BuildHasherDefault::<SeaHasher>, b);
    }

    #[bench]
    fn hashmap_fnv_lazy_static(b: &mut Bencher) {
        hashmap_lazy_static!(FnvBuildHasher, b);
    }

    #[bench]
    fn btreemap(b: &mut Bencher) {
        let samples = samples();
        let samples = &samples;
        let map: BTreeMap<_, _> = ENTITIES.iter().cloned().collect();
        let map = &map;
        b.iter(|| for sample in samples {
            test::black_box(map.get(&sample.deref()));
        });
    }

    #[bench]
    fn phf(b: &mut Bencher) {
        let samples = samples();
        let samples = &samples;
        b.iter(|| for sample in samples {
            let sample: &[u8] = sample;
            test::black_box(ENTITIES_PHF.get(sample));
        });
    }

    #[bench]
    fn x_create_hashmap_fnv(b: &mut Bencher) {
        b.iter(|| {
            let mut map = HashMap::with_capacity_and_hasher(ENTITIES.len(), FnvBuildHasher::default());
            map.extend(ENTITIES.iter().cloned());
            map
        })
    }

    #[bench]
    fn x_create_hashmap_siphash(b: &mut Bencher) {
        b.iter(|| {
            let mut map = HashMap::with_capacity_and_hasher(ENTITIES.len(), RandomState::new());
            map.extend(ENTITIES.iter().cloned());
            map
        })
    }
}

fn main() {
    println!("Hello, world!");
}
