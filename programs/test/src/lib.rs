use anchor_lang::prelude::{
    borsh::{BorshDeserialize, BorshSerialize},
    *,
};

declare_id!("9adXJRdvrX5BX7U5z5BAcNUtwmM8KTxQUa45XXwZnWm6");

#[program]
pub mod test {
    use anchor_lang::solana_program::{instruction::Instruction, program::invoke_signed};

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, fee: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.fees = fee;
        Ok(())
    }

    pub fn call_receiver_method<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CallReceiverMethod<'info>>,
        message: String,
    ) -> Result<()> {
        increment_sn(&mut ctx.accounts.state);

        let args = SendMessageArgs {
            msg: message.into_bytes(),
        };
        let mut data = vec![];
        args.serialize(&mut data).unwrap();

        let mut instruction_data = Vec::new();
        instruction_data.extend_from_slice(&sighash("global", "send_message"));
        instruction_data.extend_from_slice(&data);

        let connection_state = &ctx.remaining_accounts[0];
        let system = ctx.accounts.system_program.to_account_info();
        let user = &ctx.accounts.state.to_account_info();

        let bumps = ctx.bumps.state;

        let signers_seeds: &[&[u8]] = &[b"state", &[bumps]];

        let i = Instruction {
            program_id: *connection_state.owner,
            accounts: vec![
                AccountMeta::new_readonly(connection_state.key(), false),
                AccountMeta::new(user.key(), true),
                AccountMeta::new_readonly(system.key(), false),
            ],
            data: instruction_data,
        };

        let account_infos = vec![connection_state.clone(), user.clone(), system.clone()];

        let _ = invoke_signed(&i, &account_infos, &[signers_seeds]);

        Ok(())
    }
}

pub fn increment_sn<'info>(state: &mut Account<'info, CallerState>) -> u64 {
    state.sequence_number += 1;
    state.sequence_number
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct SendMessageArgs {
    pub msg: Vec<u8>,
}

pub fn sighash(namespace: &str, name: &str) -> Vec<u8> {
    let preimage = format!("{}:{}", namespace, name);

    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(
        &anchor_lang::solana_program::hash::hash(preimage.as_bytes()).to_bytes()[..8],
    );
    sighash.to_vec()
}

#[account]
pub struct CallerState {
    pub sequence_number: u64,
    pub fees: u64,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8 + 8, seeds = [b"state"], bump)]
    pub state: Account<'info, CallerState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CallReceiverMethod<'info> {
    #[account(mut, seeds = [b"state"], bump)]
    pub state: Account<'info, CallerState>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}
