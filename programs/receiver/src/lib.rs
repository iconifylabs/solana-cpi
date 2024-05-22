use anchor_lang::prelude::*;

declare_id!("4igfRoKzMoR5SS11KRACcnL6MhSpiJ6BQvenSKPjCTMG");

#[program]
pub mod receiver {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, xcall: Pubkey) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.xcall = xcall;
        Ok(())
    }

    pub fn send_message(ctx: Context<SendMessage>, message: Vec<u8>) -> Result<()> {
        require_keys_eq!(ctx.accounts.user.key(), ctx.accounts.state.xcall, ErrorCode::Unauthorized);
        assert!(ctx.accounts.user.is_signer);
        let msg = hex::encode(message.clone());
        msg!("[Receiver] : Sending message : {}", msg);
        Ok(())
    }
}

#[account]
pub struct ReceiverState {
    pub xcall: Pubkey,
}

#[account]
pub struct MsgAccount {
    pub msg: Vec<u8>,
}

#[derive(Accounts)]
pub struct SendMessage<'info>{
    pub state: Account<'info, ReceiverState>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,

}


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8 + 64, seeds = [b"state"], bump)]
    pub state: Account<'info, ReceiverState>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
}