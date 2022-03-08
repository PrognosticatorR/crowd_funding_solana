use crate::state::{CampaignDetails, WithdrawRequest};
use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;
/// Instructions supported by the campaign program.
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum CampaignInstruction {
    CreateCampaign(CampaignDetails),
    Withdraw(WithdrawRequest),
    Donate,
}

impl CampaignInstruction {
    pub fn unpack(instruction_data: &[u8]) -> Result<Self, ProgramError> {
        let (tag, data) = instruction_data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        match tag {
            1 => Ok(CampaignInstruction::CreateCampaign(
                CampaignDetails::try_from_slice(data)
                    .expect("can't perform deserialization for createCampaign"),
            )),
            2 => Ok(CampaignInstruction::Withdraw(
                WithdrawRequest::try_from_slice(data)
                    .expect("can't perform deserialization for withdraw"),
            )),
            3 => Ok(CampaignInstruction::Donate),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
