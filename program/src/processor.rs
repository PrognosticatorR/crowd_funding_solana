use crate::{
    instruction::CampaignInstruction,
    state::{CampaignDetails, WithdrawRequest},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = CampaignInstruction::unpack(instruction_data)?;
        match instruction {
            CampaignInstruction::CreateCampaign(data) => {
                Self::process_create_campaign(program_id, accounts, data)
            }
            CampaignInstruction::Withdraw(data) => {
                Self::process_withdraw(program_id, accounts, data)
            }
            CampaignInstruction::Donate => Self::process_donate(program_id, accounts),
        }
    }
    fn process_create_campaign(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        mut input_data: CampaignDetails,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let writing_account = next_account_info(accounts_iter)?;
        let creator_account = next_account_info(accounts_iter)?;

        // to allow transaction we want to creator account to be the signer
        if !creator_account.is_signer {
            msg!("creator_account should be signer");
            return Err(ProgramError::IncorrectProgramId);
        }
        if writing_account.owner != program_id {
            msg!("writing_account isn't owned by program");
            return Err(ProgramError::IncorrectProgramId);
        }
        if input_data.admin != *creator_account.key {
            msg!("Invaild instruction data");
            return Err(ProgramError::InvalidInstructionData);
        }

        let rent_exemption = Rent::get()?.minimum_balance(writing_account.data_len());
        if **writing_account.lamports.borrow() < rent_exemption {
            msg!("The balance of writing_account should be more then rent_exemption");
            return Err(ProgramError::InsufficientFunds);
        }
        input_data.amount_donated = 0;
        input_data.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;
        Ok(())
    }

    fn process_withdraw(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        input_data: WithdrawRequest,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let writing_account = next_account_info(accounts_iter)?;
        let admin_account = next_account_info(accounts_iter)?;

        if writing_account.owner != program_id {
            msg!("writing_account isn't owned by program");
            return Err(ProgramError::IncorrectProgramId);
        }
        if !admin_account.is_signer {
            msg!("admin should be signer");
            return Err(ProgramError::IncorrectProgramId);
        }

        let campaign_data = CampaignDetails::try_from_slice(*writing_account.data.borrow())
            .expect("Error deserializing data");

        if campaign_data.admin != *admin_account.key {
            msg!("Only the account admin can withdraw");
            return Err(ProgramError::InvalidAccountData);
        }

        let rent_exemption = Rent::get()?.minimum_balance(writing_account.data_len());
        if **writing_account.lamports.borrow() - rent_exemption < input_data.amount {
            msg!("Insufficent balance");
            return Err(ProgramError::InsufficientFunds);
        }
        **writing_account.try_borrow_mut_lamports()? -= input_data.amount;
        **admin_account.try_borrow_mut_lamports()? += input_data.amount;
        Ok(())
    }

    fn process_donate(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let writing_account = next_account_info(accounts_iter)?;
        let donator_program_account = next_account_info(accounts_iter)?;
        let donator = next_account_info(accounts_iter)?;

        if writing_account.owner != program_id {
            msg!("writing_account isn't owned by program");
            return Err(ProgramError::IncorrectProgramId);
        }

        if donator_program_account.owner != program_id {
            msg!("donator_program_account isn't owned by program");
            return Err(ProgramError::IncorrectProgramId);
        }

        if !donator.is_signer {
            msg!("donator should be signer");
            return Err(ProgramError::IncorrectProgramId);
        }

        let mut campaign_data = CampaignDetails::try_from_slice(*writing_account.data.borrow())
            .expect("Error deserializing data");
        campaign_data.amount_donated += **donator_program_account.lamports.borrow();

        **writing_account.try_borrow_mut_lamports()? += **donator_program_account.lamports.borrow();
        **donator_program_account.try_borrow_mut_lamports()? = 0;

        campaign_data.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;
        Ok(())
    }
}
