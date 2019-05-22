#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {
    use test::Bencher;
    use rayon::slice::{ParallelSliceMut, ParallelSlice};
    use rayon::iter::{IndexedParallelIterator, ParallelIterator};

    const N: usize = 1_000_000;
    const BATCH: usize = 10_000;

    fn get_data() -> (Vec<u64>, Vec<u64>) {
        let x = vec![0; N];
        let y = (0..N as u64).collect::<Vec<_>>();
        (x, y)
    }

    #[bench]
    fn rayon_memcpy(b: &mut Bencher) {
        let (mut x, y) = get_data();
        b.iter(|| {
            x.par_chunks_mut(BATCH)
                .zip(y.par_chunks(BATCH))
                .for_each(|(x, y)| x.copy_from_slice(y));
        });
    }

    #[bench]
    fn single_threaded_memcpy(b: &mut Bencher) {
        let (mut x, y) = get_data();
        b.iter(|| {
            x.copy_from_slice(&y);
        });
    }
}

fn main() {
    println!("Hello, world!");
}
