/// BN128 Addition, Scalar Multiplication and Pairing operations
///
/// Adpted from the frontier precompile:
/// https://github.com/paritytech/frontier/blob/master/frame/evm/precompile/bn128/src/lib.rs
use bn::{pairing_batch, AffineG1, AffineG2, Fq, Fq2, Fr, Group, Gt, G1, G2};

use crate::chain_ext::InvalidArgument;

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

fn read_fr(input: &[u8], start_inx: usize) -> Result<Fr, InvalidArgument> {
	let mut buf = [0u8; 32];
	read_input(input, &mut buf, start_inx);

	Ok(bn::Fr::from_slice(&buf)?)
}

fn read_point(input: &[u8], start_inx: usize) -> Result<G1, InvalidArgument> {
	let mut px_buf = [0u8; 32];
	let mut py_buf = [0u8; 32];
	read_input(input, &mut px_buf, start_inx);
	read_input(input, &mut py_buf, start_inx + 32);

	let px = Fq::from_slice(&px_buf)?;
	let py = Fq::from_slice(&py_buf)?;

	if px == Fq::zero() && py == Fq::zero() {
		Ok(G1::zero())
	} else {
		Ok(AffineG1::new(px, py)?.into())
	}
}

fn write_point(output: &mut [u8; 64], point: AffineG1) {
	let mut buf = [0; 32];
	point.x().to_big_endian(&mut buf).expect("buffer size is 32; qed");
	buf.reverse();
	output[..32].copy_from_slice(&buf);

	let mut buf = [0; 32];
	point.y().to_big_endian(&mut buf).expect("buffer size is 32; qed");
	buf.reverse();
	output[32..].copy_from_slice(&buf);
}

pub(crate) fn add(input: &[u8]) -> Result<[u8; 64], InvalidArgument> {
	let p1 = read_point(input, 0)?;
	let p2 = read_point(input, 64)?;

	let mut output = [0u8; 64];
	if let Some(point) = AffineG1::from_jacobian(p1 + p2) {
		// point not at infinity
		write_point(&mut output, point);
	}
	Ok(output)
}

pub(crate) fn mul(input: &[u8]) -> Result<[u8; 64], InvalidArgument> {
	let p = read_point(input, 0)?;
	let fr = read_fr(input, 64)?;

	let mut output = [0u8; 64];
	if let Some(point) = AffineG1::from_jacobian(p * fr) {
		// point not at infinity
		write_point(&mut output, point)
	}
	Ok(output)
}

pub(crate) fn pairing(input: &[u8]) -> Result<bool, InvalidArgument> {
	// TODO / FIXME: Fixed input size
	//if input.is_empty() {
	//	return Ok(false);
	//}
	//assert!(input.len() % 192 = 0, "bad elliptic curve pairing size");

	// (a, b_a, b_b - each 64-byte affine coordinates)
	let elements = input.len() / 192;

	let read_buf = |idx: usize, off: usize| {
		let mut buf = [0; 32];
		for (i, b) in input[idx * 192 + off..idx * 192 + off + 32].iter().rev().enumerate() {
			buf[i] = *b
		}
		buf
	};

	let mut vals = crate::Vec::new();
	for idx in 0..elements {
		let a_x = Fq::from_slice(&read_buf(idx, 0))?;
		let a_y = Fq::from_slice(&read_buf(idx, 32))?;
		let b_a_y = Fq::from_slice(&read_buf(idx, 64))?;
		let b_a_x = Fq::from_slice(&read_buf(idx, 96))?;
		let b_b_y = Fq::from_slice(&read_buf(idx, 128))?;
		let b_b_x = Fq::from_slice(&read_buf(idx, 160))?;

		let b_a = Fq2::new(b_a_x, b_a_y);
		let b_b = Fq2::new(b_b_x, b_b_y);
		let b = if b_a.is_zero() && b_b.is_zero() {
			G2::zero()
		} else {
			G2::from(AffineG2::new(b_a, b_b)?)
		};
		let a = if a_x.is_zero() && a_y.is_zero() {
			G1::zero()
		} else {
			G1::from(AffineG1::new(a_x, a_y)?)
		};
		vals.push((a, b));
	}

	Ok(pairing_batch(&vals) == Gt::one())
}
