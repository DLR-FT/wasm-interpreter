//! Benchmarks for different implementations for reading variable-length 32-bit
//! integers. Other integer sizes are not benchmarked as of now.

use std::hint::unreachable_unchecked;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn criterion_benchmark(c: &mut Criterion) {
    let samples_by_num_bytes: &[&[u8]] = &[
        &[0x7b],
        &[0x8b, 0x44],
        &[0xa3, 0x8d, 0x3c],
        &[0xab, 0x90, 0x98, 0x04],
        &[0xe8, 0x9a, 0xfb, 0x8f, 0x03],
    ];

    let mut group = c.benchmark_group("read_var_u32");

    for sample in samples_by_num_bytes {
        let len = sample.len();
        let input: &[u8] = sample;
        group.throughput(criterion::Throughput::Bytes(len as u64));

        group.bench_with_input(
            BenchmarkId::new("unbounded_loop", format!("{len}b")),
            &input,
            |b, bytes| {
                b.iter(|| {
                    let mut idx = std::hint::black_box(0);
                    let result = unbounded_loop(bytes, &mut idx);
                    (result, idx)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("unbounded_loop_unchecked", format!("{len}b")),
            &input,
            |b, bytes| {
                b.iter(|| {
                    let mut idx = std::hint::black_box(0);
                    let result = unsafe { unbounded_loop_unchecked(bytes, &mut idx) };
                    (result, idx)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("bounded_loop", format!("{len}b")),
            &input,
            |b, bytes| {
                b.iter(|| {
                    let mut idx = std::hint::black_box(0);
                    let result = bounded_loop(bytes, &mut idx);
                    (result, idx)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("bounded_loop_unchecked", format!("{len}b")),
            &input,
            |b, bytes| {
                b.iter(|| {
                    let mut idx = std::hint::black_box(0);
                    let result = unsafe { bounded_loop_unchecked(bytes, &mut idx) };
                    (result, idx)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("unrolled", format!("{len}b")),
            &input,
            |b, bytes| {
                b.iter(|| {
                    let mut idx = std::hint::black_box(0);
                    let result = unrolled(bytes, &mut idx);
                    (result, idx)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("unrolled_unchecked", format!("{len}b")),
            &input,
            |b, bytes| {
                b.iter(|| {
                    let mut idx = std::hint::black_box(0);
                    let result = unsafe { unrolled_unchecked(bytes, &mut idx) };
                    (result, idx)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("branchless_unchecked", format!("{len}b")),
            &input,
            |b, bytes| {
                b.iter(|| {
                    let mut idx = std::hint::black_box(0);
                    let result = unsafe { branchless_unchecked(bytes, &mut idx) };
                    (result, idx)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("branchless_unchecked2", format!("{len}b")),
            &input,
            |b, bytes| {
                b.iter(|| {
                    let mut idx = std::hint::black_box(0);
                    let result = unsafe { branchless_unchecked2(bytes, &mut idx) };
                    (result, idx)
                })
            },
        );

        // The exact function for the optimal scenario is chosen based on the sample length
        group.bench_with_input(
            BenchmarkId::new("optimal", format!("{len}b")),
            &input,
            |b, bytes| match len {
                1 => b.iter(|| {
                    let mut idx = std::hint::black_box(0);
                    let result = unsafe { optimal::one_byte_unchecked(bytes, &mut idx) };
                    (result, idx)
                }),
                2 => b.iter(|| {
                    let mut idx = std::hint::black_box(0);
                    let result = unsafe { optimal::two_bytes_unchecked(bytes, &mut idx) };
                    (result, idx)
                }),
                3 => b.iter(|| {
                    let mut idx = std::hint::black_box(0);
                    let result = unsafe { optimal::three_bytes_unchecked(bytes, &mut idx) };
                    (result, idx)
                }),
                4 => b.iter(|| {
                    let mut idx = std::hint::black_box(0);
                    let result = unsafe { optimal::four_bytes_unchecked(bytes, &mut idx) };
                    (result, idx)
                }),
                5 => b.iter(|| {
                    let mut idx = std::hint::black_box(0);
                    let result = unsafe { optimal::five_bytes_unchecked(bytes, &mut idx) };
                    (result, idx)
                }),
                _ => {
                    unimplemented!("no optimal function available for sample length {len}");
                }
            },
        );
    }

    group.finish();
}

#[inline(always)]
fn read_u8(wasm: &[u8], i: &mut usize) -> Option<u8> {
    wasm.get(*i).copied().inspect(|_| *i += 1)
}

/// Reads a single byte
///
/// # Safety
/// The caller must guarantee that there is a u8 in the wasm at the given index
#[inline(always)]
unsafe fn read_u8_unchecked(wasm: &[u8], i: &mut usize) -> u8 {
    // Safety: Guaranteed by the caller
    let v = unsafe { wasm.get(*i).unwrap_unchecked() };
    *i += 1;
    *v
}

/// Reads a variable-length u32 using a `loop`.
#[inline(always)]
fn unbounded_loop(wasm: &[u8], i: &mut usize) -> Option<u32> {
    const CONTINUATION_BIT: u8 = 1 << 7;

    let mut result: u32 = 0;
    let mut shift = 0;

    loop {
        let byte = read_u8(wasm, i)?;
        result |= ((byte & 0x7F) as u32) << shift;
        // maximum allowed num of leb bytes for u32 is ceil(32.0/7.0) == 5
        // shift >= 28 checks we're at the 5th bit or larger
        // byte >> 32-28 checks whether (this byte lost bits when shifted) or (the continuation bit is set)
        if shift >= 28 && byte >> (32 - shift) != 0 {
            return None;
        }

        if byte & CONTINUATION_BIT == 0 {
            return Some(result);
        }

        shift += 7;
    }
}

/// Reads a variable-length u32 using a `loop`.
///
/// # Safety
/// The caller must guarantee that there is a valid variable-length u32 at the given index.
#[inline(always)]
unsafe fn unbounded_loop_unchecked(wasm: &[u8], i: &mut usize) -> u32 {
    const CONTINUATION_BIT: u8 = 1 << 7;

    let mut result: u32 = 0;
    let mut shift = 0;

    loop {
        let byte = unsafe { read_u8_unchecked(wasm, i) };
        result |= ((byte & 0x7F) as u32) << shift;
        // maximum allowed num of leb bytes for u32 is ceil(32.0/7.0) == 5
        // shift >= 28 checks we're at the 5th bit or larger
        // byte >> 32-28 checks whether (this byte lost bits when shifted) or (the continuation bit is set)
        if shift >= 28 && byte >> (32 - shift) != 0 {
            unsafe { unreachable_unchecked() }
        }

        if byte & CONTINUATION_BIT == 0 {
            return result;
        }

        shift += 7;
    }
}

/// Reads a variable-length u32 using a bounded `for` loop.
#[inline(always)]
fn bounded_loop(wasm: &[u8], i: &mut usize) -> Option<u32> {
    const CONTINUATION_BIT: u8 = 1 << 7;

    let mut result: u32 = 0;
    let mut shift = 0;

    for _ in 0..5 {
        let byte = read_u8(wasm, i)?;
        result |= ((byte & 0x7F) as u32) << shift;
        // maximum allowed num of leb bytes for u32 is ceil(32.0/7.0) == 5
        // shift >= 28 checks we're at the 5th bit or larger
        // byte >> 32-28 checks whether (this byte lost bits when shifted) or (the continuation bit is set)
        if shift >= 28 && byte >> (32 - shift) != 0 {
            return None;
        }

        if byte & CONTINUATION_BIT == 0 {
            return Some(result);
        }

        shift += 7;
    }

    None
}

/// Reads a variable-length u32 using a bounded `for` loop.
///
/// # Note
/// At the time of writing, the compiler does not unroll this loop which makes
/// this function slower than [`bounded_loop`].
///
/// # Safety
/// The caller must guarantee that there is a valid variable-length u32 at the given index.
#[inline(always)]
unsafe fn bounded_loop_unchecked(wasm: &[u8], i: &mut usize) -> u32 {
    const CONTINUATION_BIT: u8 = 1 << 7;

    let mut result: u32 = 0;
    let mut shift = 0;

    for _ in 0..5 {
        let byte = unsafe { read_u8_unchecked(wasm, i) };
        result |= ((byte & 0x7F) as u32) << shift;
        // maximum allowed num of leb bytes for u32 is ceil(32.0/7.0) == 5
        // shift >= 28 checks we're at the 5th bit or larger
        // byte >> 32-28 checks whether (this byte lost bits when shifted) or (the continuation bit is set)
        if shift >= 28 && byte >> (32 - shift) != 0 {
            unsafe { unreachable_unchecked() }
        }

        if byte & CONTINUATION_BIT == 0 {
            return result;
        }

        shift += 7;
    }

    unsafe { unreachable_unchecked() }
}

/// Reads a variable-length u32 without a loop.
#[inline(always)]
fn unrolled(wasm: &[u8], i: &mut usize) -> Option<u32> {
    let mut result: u32 = 0;

    let byte = read_u8(wasm, i)?;
    result |= (byte & 0x7F) as u32;

    if byte & 0x80 == 0 {
        return Some(result);
    }

    let byte = read_u8(wasm, i)?;
    result |= ((byte & 0x7F) as u32) << 7;
    if byte & 0x80 == 0 {
        return Some(result);
    }

    let byte = read_u8(wasm, i)?;
    result |= ((byte & 0x7F) as u32) << 14;
    if byte & 0x80 == 0 {
        return Some(result);
    }

    let byte = read_u8(wasm, i)?;
    result |= ((byte & 0x7F) as u32) << 21;
    if byte & 0x80 == 0 {
        return Some(result);
    }

    let byte = read_u8(wasm, i)?;
    result |= ((byte & 0x7F) as u32) << 28;

    Some(result)
}

/// Reads a variable-length u32 without a loop.
///
/// # Safety
/// The caller must guarantee that there is a valid variable-length u32 at the given index.
#[inline(always)]
unsafe fn unrolled_unchecked(wasm: &[u8], i: &mut usize) -> u32 {
    let mut result: u32 = 0;

    let byte = unsafe { read_u8_unchecked(wasm, i) };
    result |= (byte & 0x7F) as u32;
    if byte & 0x80 == 0 {
        return result;
    }

    let byte = unsafe { read_u8_unchecked(wasm, i) };
    result |= ((byte & 0x7F) as u32) << 7;
    if byte & 0x80 == 0 {
        return result;
    }

    let byte = unsafe { read_u8_unchecked(wasm, i) };
    result |= ((byte & 0x7F) as u32) << 14;
    if byte & 0x80 == 0 {
        return result;
    }

    let byte = unsafe { read_u8_unchecked(wasm, i) };
    result |= ((byte & 0x7F) as u32) << 21;
    if byte & 0x80 == 0 {
        return result;
    }

    let byte = unsafe { read_u8_unchecked(wasm, i) };
    result |= ((byte & 0x7F) as u32) << 28;

    result
}

/// Reads a variable-length u32 without a loop or branches.
///
/// # Implementation Details
/// A total of five memory accesses must be made with a branchless
/// implementation. This implementation saturates the read index at the current
/// read index if the `0x80` bitflag of the current byte is zero, i.e. if this
/// was the last byte belonging to the number currently being read.
///
/// # Safety
/// The caller must guarantee that there is a valid variable-length u32 at the given index.
#[inline(always)]
unsafe fn branchless_unchecked(wasm: &[u8], i: &mut usize) -> u32 {
    let byte1 = unsafe { read_u8_unchecked(wasm, i) };
    let next_bit1 = byte1 & 0x80;
    *i -= (!next_bit1 >> 7) as usize;
    let mut result = (byte1 & 0x7F) as u32;

    let byte2 = unsafe { read_u8_unchecked(wasm, i) };
    let next_bit2 = byte2 & 0x80;
    *i -= (!next_bit2 >> 7) as usize;
    result |= ((byte2 & !next_bit1) as u32) << 7;

    let byte3 = unsafe { read_u8_unchecked(wasm, i) };
    let next_bit3 = byte3 & 0x80;
    *i -= (!next_bit3 >> 7) as usize;
    result |= ((byte3 & !next_bit2) as u32) << 14;

    let byte4 = unsafe { read_u8_unchecked(wasm, i) };
    let next_bit4 = byte4 & 0x80;
    *i -= (!next_bit4 >> 7) as usize;
    result |= ((byte4 & !next_bit3) as u32) << 21;

    let byte5 = unsafe { read_u8_unchecked(wasm, i) };
    result |= ((byte5 & !next_bit4) as u32) << 28;

    result
}

/// Reads a variable-length u32 without a loop or branches.
///
/// # Implementation Details
/// A total of five memory accesses must be made with a branchless
/// implementation. This implementation saturates the read index at the last
/// element of the `wasm` byte slice.
///
/// # Safety
/// The caller must guarantee that there is a valid variable-length u32 at the given index.
#[inline(always)]
unsafe fn branchless_unchecked2(wasm: &[u8], i: &mut usize) -> u32 {
    let len = wasm.len();
    if len == 0 {
        unsafe {
            std::hint::unreachable_unchecked();
        }
    }

    let i1 = *i;
    let i2 = (*i + 1).min(len - 1);
    let i3 = (*i + 2).min(len - 1);
    let i4 = (*i + 3).min(len - 1);
    let i5 = (*i + 4).min(len - 1);

    let byte1 = unsafe { wasm.get(i1).unwrap_unchecked() };
    let next_bit1 = byte1 & 0x80;
    let mut result = (byte1 & 0x7F) as u32;

    let byte2 = unsafe { wasm.get(i2).unwrap_unchecked() };
    let next_bit2 = next_bit1 & byte2;
    result |= ((byte2 & !next_bit1) as u32) << 7;

    let byte3 = unsafe { wasm.get(i3).unwrap_unchecked() };
    let next_bit3 = next_bit2 & byte3;
    result |= ((byte3 & !next_bit2) as u32) << 14;

    let byte4 = unsafe { wasm.get(i4).unwrap_unchecked() };
    let next_bit4 = next_bit3 & byte4;
    result |= ((byte4 & !next_bit3) as u32) << 21;

    let byte5 = unsafe { wasm.get(i5).unwrap_unchecked() };
    result |= ((byte5 & !next_bit4) as u32) << 28;

    *i += 1
        + ((next_bit1 as usize + next_bit2 as usize + next_bit3 as usize + next_bit4 as usize)
            >> 7);

    result
}

/// These read functions are optimal for every possible length.
mod optimal {
    use crate::read_u8_unchecked;

    /// # Safety
    /// The caller must guarantee that there is a valid variable-length u32 with a length of one byte at the given index.
    #[inline(always)]
    pub unsafe fn one_byte_unchecked(wasm: &[u8], i: &mut usize) -> u32 {
        // Safety: Guaranteed by the caller
        unsafe { read_u8_unchecked(wasm, i) as u32 }
    }

    /// # Safety
    /// The caller must guarantee that there is a valid variable-length u32 with a length of two bytes at the given index.
    #[inline(always)]
    pub unsafe fn two_bytes_unchecked(wasm: &[u8], i: &mut usize) -> u32 {
        // Safety: Guaranteed by the caller
        let v1 = unsafe { read_u8_unchecked(wasm, i) } as u32;
        let v2 = unsafe { read_u8_unchecked(wasm, i) } as u32;

        v1 as u32 & 0x7F | v2 << 7
    }

    /// # Safety
    /// The caller must guarantee that there is a valid variable-length u32 with a length of three bytes at the given index.
    #[inline(always)]
    pub unsafe fn three_bytes_unchecked(wasm: &[u8], i: &mut usize) -> u32 {
        // Safety: Guaranteed by the caller
        let v1 = unsafe { read_u8_unchecked(wasm, i) } as u32;
        let v2 = unsafe { read_u8_unchecked(wasm, i) } as u32;
        let v3 = unsafe { read_u8_unchecked(wasm, i) } as u32;

        v1 as u32 & 0x7F | v2 & 0x7F << 7 | v3 << 14
    }

    /// # Safety
    /// The caller must guarantee that there is a valid variable-length u32 with a length of four bytes at the given index.
    #[inline(always)]
    pub unsafe fn four_bytes_unchecked(wasm: &[u8], i: &mut usize) -> u32 {
        // Safety: Guaranteed by the caller
        let v1 = unsafe { read_u8_unchecked(wasm, i) } as u32;
        let v2 = unsafe { read_u8_unchecked(wasm, i) } as u32;
        let v3 = unsafe { read_u8_unchecked(wasm, i) } as u32;
        let v4 = unsafe { read_u8_unchecked(wasm, i) } as u32;

        v1 as u32 & 0x7F | v2 & 0x7F << 7 | v3 & 0x7F << 14 | v4 << 21
    }

    /// # Safety
    /// The caller must guarantee that there is a valid variable-length u32 with a length of five bytes at the given index.
    #[inline(always)]
    pub unsafe fn five_bytes_unchecked(wasm: &[u8], i: &mut usize) -> u32 {
        // Safety: Guaranteed by the caller
        let v1 = unsafe { read_u8_unchecked(wasm, i) } as u32;
        let v2 = unsafe { read_u8_unchecked(wasm, i) } as u32;
        let v3 = unsafe { read_u8_unchecked(wasm, i) } as u32;
        let v4 = unsafe { read_u8_unchecked(wasm, i) } as u32;
        let v5 = unsafe { read_u8_unchecked(wasm, i) } as u32;

        v1 as u32 & 0x7F | v2 & 0x7F << 7 | v3 & 0x7F << 14 | v4 & 0x7F << 21 | v5 << 28
    }
}
