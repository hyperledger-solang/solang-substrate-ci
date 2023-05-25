/// BN128 Addition, Scalar Multiplication and Pairing operations
///
/// Adpted from the frontier precompile:
/// https://github.com/paritytech/frontier/blob/master/frame/evm/precompile/bn128/src/lib.rs
use crate::Vec;

/// Copy bytes from input to target.
fn read_input(source: &[u8], target: &mut [u8], offset: usize) {
	// Out of bounds, nothing to copy.
	if source.len() <= offset {
		return;
	}

	// Find len to copy up to target len, but not out of bounds.
	let len = core::cmp::min(target.len(), source.len() - offset);
	target[..len].copy_from_slice(&source[offset..][..len]);
	target.reverse();
}

fn read_fr(input: &[u8], start_inx: usize) -> bn::Fr {
	let mut buf = [0u8; 32];
	read_input(input, &mut buf, start_inx);

	bn::Fr::from_slice(&buf).expect("Invalid field element")
}

fn read_point(input: &[u8], start_inx: usize) -> bn::G1 {
	use bn::{AffineG1, Fq, Group, G1};

	let mut px_buf = [0u8; 32];
	let mut py_buf = [0u8; 32];
	read_input(input, &mut px_buf, start_inx);
	read_input(input, &mut py_buf, start_inx + 32);

	let px = Fq::from_slice(&px_buf).expect("Invalid point x coordinate");

	let py = Fq::from_slice(&py_buf).expect("Invalid point y coordinate");

	if px == Fq::zero() && py == Fq::zero() {
		G1::zero()
	} else {
		AffineG1::new(px, py).expect("Invalid curve point").into()
	}
}

pub(crate) fn add(input: &[u8]) -> [u8; 64] {
	use bn::AffineG1;

	let p1 = read_point(input, 0);
	let p2 = read_point(input, 64);

	let mut output = [0u8; 64];
	if let Some(sum) = AffineG1::from_jacobian(p1 + p2) {
		// point not at infinity
		let mut buf = [0; 32];
		sum.x()
			.to_big_endian(&mut buf)
			.expect("Cannot fail since 0..32 is 32-byte length");
		buf.reverse();
		output[..32].copy_from_slice(&buf);

		let mut buf = [0; 32];
		sum.y()
			.to_big_endian(&mut buf)
			.expect("Cannot fail since 32..64 is 32-byte length");
		buf.reverse();
		output[32..].copy_from_slice(&buf);
	}

	output
}

pub(crate) fn mul(input: &[u8]) -> [u8; 64] {
	use bn::AffineG1;

	let p = read_point(input, 0);
	let fr = read_fr(input, 64);

	let mut output = [0u8; 64];
	if let Some(sum) = AffineG1::from_jacobian(p * fr) {
		// point not at infinity
		let mut buf = [0; 32];
		sum.x()
			.to_big_endian(&mut buf)
			.expect("Cannot fail since 0..32 is 32-byte length");
		buf.reverse();
		output[..32].copy_from_slice(&buf);

		let mut buf = [0; 32];
		sum.y()
			.to_big_endian(&mut buf)
			.expect("Cannot fail since 32..64 is 32-byte length");
		buf.reverse();
		output[32..].copy_from_slice(&buf);
	}
	output
}

pub(crate) fn pairing(input: &[u8]) -> bool {
	use bn::{pairing_batch, AffineG1, AffineG2, Fq, Fq2, Group, Gt, G1, G2};

	if input.is_empty() {
		return true;
	}
	if input.len() % 192 > 0 {
		panic!("bad elliptic curve pairing size");
	}

	// (a, b_a, b_b - each 64-byte affine coordinates)
	let elements = input.len() / 192;

	let read_buf = |idx: usize, off: usize| {
		let mut buf = [0; 32];
		for (i, b) in input[idx * 192 + off..idx * 192 + off + 32].iter().rev().enumerate() {
			buf[i] = *b
		}
		buf
	};

	let mut vals = Vec::new();
	for idx in 0..elements {
		let a_x = Fq::from_slice(&read_buf(idx, 0)).expect("Invalid a argument x coordinate");

		let a_y = Fq::from_slice(&read_buf(idx, 32)).expect("Invalid a argument y coordinate");

		let b_a_y = Fq::from_slice(&read_buf(idx, 64))
			.expect("Invalid b argument imaginary coeff x coordinate");

		let b_a_x = Fq::from_slice(&read_buf(idx, 96))
			.expect("Invalid b argument imaginary coeff y coordinate");

		let b_b_y = Fq::from_slice(&read_buf(idx, 128))
			.expect("Invalid b argument real coeff x coordinate");

		let b_b_x = Fq::from_slice(&read_buf(idx, 160))
			.expect("Invalid b argument real coeff y coordinate");

		let b_a = Fq2::new(b_a_x, b_a_y);
		let b_b = Fq2::new(b_b_x, b_b_y);
		let b = if b_a.is_zero() && b_b.is_zero() {
			G2::zero()
		} else {
			G2::from(AffineG2::new(b_a, b_b).expect("Invalid b argument - not on curve"))
		};
		let a = if a_x.is_zero() && a_y.is_zero() {
			G1::zero()
		} else {
			G1::from(AffineG1::new(a_x, a_y).expect("Invalid a argument - not on curve"))
		};
		vals.push((a, b));
	}

	pairing_batch(&vals) == Gt::one()
}
