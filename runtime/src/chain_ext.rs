use bn::{FieldError, GroupError};
use codec::Encode;
use ff_wasm_unknown_unknown::PrimeField;
use frame_support::weights::Weight;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::DispatchError;

use crate::{mimc::mimc_feistel, Runtime};
use frame_support::traits::Randomness;

pub(crate) enum InvalidArgument {
	NotInField = 1,
	NotOnCurve = 2,
}

impl From<FieldError> for InvalidArgument {
	fn from(_: FieldError) -> Self {
		Self::NotInField
	}
}

impl From<GroupError> for InvalidArgument {
	fn from(_: GroupError) -> Self {
		Self::NotOnCurve
	}
}

#[derive(Default)]
pub struct FetchRandomExtension;

impl ChainExtension<Runtime> for FetchRandomExtension {
	fn call<E: Ext>(&mut self, mut env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		let func_id = env.func_id();
		match func_id {
			// ink! FetchRandom chain extension example
			1101 => {
				//debug!(
				//	target: "runtime",
				//	"[ChainExtension]|call|func_id:{:}",
				//	func_id
				//);
				let mut env = env.buf_in_buf_out();
				let arg: [u8; 32] = env.read_as()?;
				let random_seed = crate::RandomnessCollectiveFlip::random(&arg).0;
				let random_slice = random_seed.encode();
				env.write(&random_slice, false, None)
					.map_err(|_| DispatchError::Other("ChainExtension failed to call random"))?;
			},

			// bn128 curve addition
			6 => {
				env.charge_weight(Weight::from_parts(7_940_000, 0))?; // Roughly

				let mut env = env.buf_in_buf_out();
				let arg: [u8; 128] = env.read_as()?;

				match crate::bn128::add(&arg) {
					Ok(result) => env
						.write(&result, false, None)
						.map_err(|_| DispatchError::Other("output buffer too small"))?,
					Err(reason) => return Ok(RetVal::Converging(reason as u32)),
				}
			},

			// bn128 curve scalar multiplication
			7 => {
				env.charge_weight(Weight::from_parts(168_074_000, 0))?; // Roughly

				let mut env = env.buf_in_buf_out();
				let arg: [u8; 96] = env.read_as()?;

				match crate::bn128::mul(&arg) {
					Ok(result) => env
						.write(&result, false, None)
						.map_err(|_| DispatchError::Other("output buffer too small"))?,
					Err(reason) => return Ok(RetVal::Converging(reason as u32)),
				}
			},

			// bn128 curve pairing
			8 => {
				env.charge_weight(Weight::from_parts(6_080_874_000, 0))?; // Roughly

				let mut env = env.buf_in_buf_out();
				let arg: [u8; 0x300] = env.read_as()?; // TOOD / FIXME: Hardcoded input size

				match crate::bn128::pairing(&arg) {
					Ok(result) => env
						.write(&result.encode(), false, None)
						.map_err(|_| DispatchError::Other("output buffer too small"))?,
					Err(reason) => return Ok(RetVal::Converging(reason as u32)),
				}
			},

			// mimc sponge hasher
			220 => {
				env.charge_weight(Weight::from_parts(28_890_000, 0))?; // Roughly

				let mut env = env.buf_in_buf_out();
				let (x_l, x_r) = env.read_as::<([u8; 32], [u8; 32])>()?;

				let result = mimc_feistel(x_l.into(), x_r.into());
				env.write(&(result.0.to_repr().0, result.1.to_repr().0).encode(), false, None)
					.map_err(|_| DispatchError::Other("output buffer too small"))?;
			},

			_ => {
				//error!("Called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			},
		}
		Ok(RetVal::Converging(0))
	}

	fn enabled() -> bool {
		true
	}
}
