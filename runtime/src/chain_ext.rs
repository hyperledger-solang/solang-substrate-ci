use codec::Encode;
use ff_wasm_unknown_unknown::PrimeField;
use frame_support::log::{debug, error};
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::DispatchError;

use super::Randomness;
use crate::{mimc::mimc_feistel, Runtime};

#[derive(Default)]
pub struct FetchRandomExtension;

impl ChainExtension<Runtime> for FetchRandomExtension {
	fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		let func_id = env.func_id();
		match func_id {
			1101 => {
				debug!(
					target: "runtime",
					"[ChainExtension]|call|func_id:{:}",
					func_id
				);
				let mut env = env.buf_in_buf_out();
				let arg: [u8; 32] = env.read_as()?;
				debug!(target: "runtime", "arg: {:?}", &arg);
				let random_seed = crate::RandomnessCollectiveFlip::random(&arg).0;
				let random_slice = random_seed.encode();
				debug!(target: "runtime", "random_slice: {:?}", &random_slice);
				env.write(&random_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call random"))?;
			},

			6 => {
				let mut env = env.buf_in_buf_out();
				let arg: [u8; 0xc0] = env.read_as()?;
				let result = crate::bn128::add(&arg);
				env.write(&result, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call bn128 add"))?;
			},

			7 => {
				let mut env = env.buf_in_buf_out();
				let arg: [u8; 0x80] = env.read_as()?;
				let result = crate::bn128::mul(&arg);
				env.write(&result, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call bn128 mul"))?;
			},

			8 => {
				let mut env = env.buf_in_buf_out();
				let arg: [u8; 0x300] = env.read_as()?; // TOOD / FIXME: Hardcoded input size
				let result = crate::bn128::pairing(&arg).encode();
				env.write(&result, false, None).map_err(|_| {
					DispatchError::Other("ChainExtension failed to call bn128 pair")
				})?;
			},

			220 => {
				let mut env = env.buf_in_buf_out();
				let (x_l, x_r) = env.read_as::<([u8; 32], [u8; 32])>()?;
				let result = mimc_feistel([0; 32].into(), x_l.into(), x_r.into());
				env.write(&(result.0.to_repr().0, result.1.to_repr().0).encode(), false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call bn128 add"))?;
			},

			_ => {
				error!("Called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			},
		}
		Ok(RetVal::Converging(0))
	}

	fn enabled() -> bool {
		true
	}
}
