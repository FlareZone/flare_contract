// SPDX-License-Identifier: Apache-2.0
// Bet.sol

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock, entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};
use std::convert::TryInto;

pub struct BetInfo {
    pub bettor: Pubkey,
    pub amount: u64,
    pub time: i64,
}

pub struct Bet {
    pub bets: Vec<BetInfo>,
    pub is_bet: bool,
    pub bet_amount: u64,
    pub bet_address: Pubkey,
    pub end_time: i64,
}

impl Bet {
    pub fn new() -> Self {
        Bet {
            bets: vec![],
            is_bet: false,
            bet_amount: 0,
            bet_address: Pubkey::default(),
            end_time: 0,
        }
    }

    pub fn publish_post(is_bet: bool, bet_amount: u64, program_id: &Pubkey) -> ProgramResult {
        let accounts = [
            AccountInfo::new(program_id.clone(), false),
            AccountInfo::new(clock::id(), false),
            user_account.clone(),
            AccountInfo::new(bet_account.key, false),
            AccountInfo::new_readonly(Sysvar::rent_id(), false),
            AccountInfo::new_readonly(Sysvar::clock_id(), false),
            AccountInfo::new_readonly(Sysvar::sysvar_id(), false),
            AccountInfo::new_readonly(bet_account.owner, false),
            AccountInfo::new_readonly_ref(&bet_account.data.borrow(), false),
        ];

        // 发布贴子，并选择是否进行对赌
        if is_bet {
            // 质押 token 到合约...
            let mut bet_account = next_account_info(accounts.iter_mut())?;
            bet_account.try_borrow_mut_data()?;
            bet_account.data.borrow_mut()[0] = 1u8;
            *bet_account.lamports.borrow_mut() -= bet_amount;

            let mut this = Self::deserialize(accounts[0].data.borrow())?;
            this.is_bet = true;
            this.bet_amount = bet_amount;
            this.bet_address = *accounts[0].key;
            this.end_time = Clock::get()?.unix_timestamp + 86400; // 24小时后结束对赌
            accounts[0].data.replace(&this.serialize());
        } else {
            let mut this = Self::deserialize(accounts[0].data.borrow())?;
            this.is_bet = false;
            accounts[0].data.replace(&this.serialize());
        }

        Ok(())
    }

    pub fn participate(
        program_id: &Pubkey,
        bet_amount: u64,
        user_account: &AccountInfo,
    ) -> ProgramResult {
        let accounts = [
            AccountInfo::new_readonly(program_id.clone(), false),
            AccountInfo::new_readonly(clock::id(), false),
            user_account.clone(),
        ];

        // 参与对赌
        let mut this = Self::deserialize(accounts[0].data.borrow())?;
        if !this.is_bet {
            return Err(ProgramError::InvalidInstructionData);
        }
        if bet_amount != this.bet_amount {
            return Err(ProgramError::InvalidInstructionData);
        }
        let mut user_account = next_account_info(accounts.iter_mut())?;
        let mut bet_account = next_account_info(accounts.iter_mut())?;
        let mut user_balance = user_account.try_borrow_mut_lamports()?;
        let mut bet_balance = bet_account.try_borrow_mut_lamports()?;
        if *user_balance < bet_amount {
            return Err(ProgramError::InsufficientFunds);
        }
        *user_balance -= bet_amount;
        *bet_balance += bet_amount;
        this.bets.push(BetInfo {
            bettor: *user_account.key,
            amount: bet_amount,
            time: Clock::get()?.unix_timestamp,
        });
        accounts[0].data.replace(&this.serialize());

        Ok(())
    }

    pub fn end_bet(program_id: &Pubkey) -> ProgramResult {
        let accounts = [
            AccountInfo::new(program_id.clone(), false),
            AccountInfo::new(clock::id(), false),
            user_account.clone(),
            AccountInfo::new(bet_account.key, false),
            AccountInfo::new_readonly(Sysvar::rent_id(), false),
            AccountInfo::new_readonly(Sysvar::clock_id(), false),
            AccountInfo::new_readonly(Sysvar::sysvar_id(), false),
            AccountInfo::new_readonly(bet_account.owner, false),
            AccountInfo::new_readonly_ref(&bet_account.data.borrow(), false),
        ];

        // 结束对赌
        let mut this = Self::deserialize(accounts[0].data.borrow())?;
        if !this.is_bet {
            return Err(ProgramError::InvalidInstructionData);
        }
        if Clock::get()?.unix_timestamp < this.end_time {
            return Err(ProgramError::InvalidInstructionData);
        }
        let total_amount = accounts[0].lamports();
        let bettor_count = this.bets.len() as u64;
        let bettor_share = total_amount / bettor_count;
        for bet in this.bets.iter() {
            let mut bettor_account = next_account_info(accounts.iter_mut())?;
            if *bettor_account.key != bet.bettor {
                return Err(ProgramError::InvalidInstructionData);
            }
            let mut bettor_balance = bettor_account.try_borrow_mut_lamports()?;
            *bettor_balance += bettor_share;
        }
        let mut bet_account = next_account_info(accounts.iter_mut())?;
        let mut bet_balance = bet_account.try_borrow_mut_lamports()?;
        *bet_balance -= total_amount;
        this.is_bet = false;
        this.bet_amount = 0;
        this.bet_address = Pubkey::default();
        this.end_time = 0;
        this.bets.clear();
        accounts[0].data.replace(&this.serialize());

        Ok(())
    }

    pub fn get_bet_info(program_id: &Pubkey) -> Result<Vec<BetInfo>, ProgramError> {
        let accounts = [AccountInfo::new_readonly(program_id.clone(), false)];

        // 查询当前帖子的投注情况
        let this = Self::deserialize(accounts[0].data.borrow())?;
        Ok(this.bets)
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(self.is_bet as u8).to_le_bytes());
        let is_bet = u8::from_le_bytes(data[0..1].try_into().unwrap()) != 0;
        buf.extend_from_slice(&self.bet_amount.to_le_bytes());
        buf.extend_from_slice(&self.bet_address.to_bytes());
        buf.extend_from_slice(&self.end_time.to_le_bytes());
        buf.extend_from_slice(&(self.bets.len() as u64).to_le_bytes());
        for bet in &self.bets {
            buf.extend_from_slice(&bet.bettor.to_bytes());
            buf.extend_from_slice(&bet.amount.to_le_bytes());
            buf.extend_from_slice(&bet.time.to_le_bytes());
        }
        buf
    }

    fn deserialize(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() < 33 {
            return Err(ProgramError::InvalidAccountData);
        }
        let is_bet = bool::from_le_bytes(data[0..1].try_into().unwrap());
        let bet_amount = u64::from_le_bytes(data[1..9].try_into().unwrap());
        let bet_address = Pubkey::new_from_array(data[9..41].try_into().unwrap());
        let end_time = i64::from_le_bytes(data[41..49].try_into().unwrap());
        let bet_count = u64::from_le_bytes(data[49..57].try_into().unwrap()) as usize;
        let mut bets = Vec::with_capacity(bet_count);
        let mut offset = 57;
        for _ in 0..bet_count {
            let bettor = Pubkey::new_from_array(data[offset..offset + 32].try_into().unwrap());
            let amount = u64::from_le_bytes(data[offset + 32..offset + 40].try_into().unwrap());
            let time = i64::from_le_bytes(data[offset + 40..offset + 48].try_into().unwrap());
            bets.push(BetInfo {
                bettor,
                amount,
                time,
            });
            offset += 48;
        }
        Ok(Bet {
            bets,
            is_bet,
            bet_amount,
            bet_address,
            end_time,
        })
    }
}

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = instruction_data[0];
    match instruction {
        0 => {
            // 发布帖子, 并选择是否进行对赌
            let is_bet = bool::from_le_bytes(instruction_data[1..2].try_into().unwrap());
            let bet_amount = u64::from_le_bytes(instruction_data[2..10].try_into().unwrap());
            Bet::publish_post(is_bet, bet_amount, program_id)
        }
        1 => {
            // 参与对赌
            let bet_amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            let user_account = next_account_info(accounts.iter())?;
            Bet::participate(program_id, bet_amount, user_account)
        }
        2 => {
            // 结束对赌
            Bet::end_bet(program_id)
        }
        3 => {
            // 查询当前帖子的投注情况
            let bet_info = Bet::get_bet_info(program_id)?;
            msg!("Bet Info:");
            for bet in bet_info {
                msg!("Bettor: {}", bet.bettor);
                msg!("Amount: {}", bet.amount);
                msg!("Time: {}", bet.time);
            }
            Ok(())
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
