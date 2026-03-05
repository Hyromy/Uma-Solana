use anchor_lang::prelude::*;

mod random;
mod racecourse;
mod uma;
mod bot;
mod race;

use uma::{Uma, StatType};
use race::prepare_to_race;

declare_id!("72PwRxpFvGCHWq6LXE5rHo7hRDcgRKNbcPd5FMxinWjp");

#[program]
pub mod uma_solana {
    use super::*;

    /// Crea una nueva Uma para el signer.
    /// Seeds: ["uma", owner_pubkey]
    pub fn create_uma(ctx: Context<CreateUma>, name: String) -> Result<()> {
        let acc   = &mut ctx.accounts.uma_account;
        acc.owner = ctx.accounts.owner.key();
        acc.uma   = Uma::new(name, true);
        acc.bump  = ctx.bumps.uma_account;
        Ok(())
    }

    /// Entrena un stat. stat_id: 0=Speed 1=Stamina 2=Power 3=Guts 4=Wit
    pub fn train(ctx: Context<GameAction>, stat_id: u8) -> Result<()> {
        let uma  = &mut ctx.accounts.uma_account.uma;
        let stat = StatType::from_u8(stat_id).ok_or(UmaError::InvalidStat)?;
        let fc   = uma.failure_chance(&stat);
        uma.train(stat, fc);
        Ok(())
    }

    /// Descansa: recupera energía, cuesta un turno de entrenamiento.
    pub fn rest(ctx: Context<GameAction>) -> Result<()> {
        ctx.accounts.uma_account.uma.rest();
        Ok(())
    }

    /// Recreación: mejora el ánimo, cuesta un turno de entrenamiento.
    pub fn recreation(ctx: Context<GameAction>) -> Result<()> {
        ctx.accounts.uma_account.uma.recreation();
        Ok(())
    }

    /// Corre la carrera programada. Los resultados se loggean con msg!().
    pub fn race(ctx: Context<GameAction>) -> Result<()> {
        let uma_acc = &mut ctx.accounts.uma_account;

        if uma_acc.uma.get_turns_to_race() > 0 {
            return Err(UmaError::CannotRaceYet.into());
        }

        let placeholder  = Uma::new_placeholder();
        let uma          = core::mem::replace(&mut uma_acc.uma, placeholder);
        let mut race_obj = prepare_to_race(uma).expect("turns_to_race already verified");
        let position     = race_obj.run();

        let mut all    = race_obj.into_runners();
        let pi         = all.iter().position(|r| r.is_human()).unwrap();
        let mut player = all.remove(pi);
        player.race_result(position);

        uma_acc.uma = player;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Cuenta
// ---------------------------------------------------------------------------

#[account]
pub struct UmaAccount {
    pub owner: Pubkey,
    pub uma:   Uma,
    pub bump:  u8,
}
impl UmaAccount {
    pub const SIZE: usize = 8 + 32 + Uma::BORSH_SIZE + 1;
}

// ---------------------------------------------------------------------------
// Contextos
// ---------------------------------------------------------------------------

#[derive(Accounts)]
pub struct CreateUma<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = UmaAccount::SIZE,
        seeds = [b"uma", owner.key().as_ref()],
        bump
    )]
    pub uma_account: Account<'info, UmaAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GameAction<'info> {
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds   = [b"uma", owner.key().as_ref()],
        bump    = uma_account.bump,
        has_one = owner,
    )]
    pub uma_account: Account<'info, UmaAccount>,
}

// ---------------------------------------------------------------------------
// Errores
// ---------------------------------------------------------------------------

#[error_code]
pub enum UmaError {
    #[msg("stat_id inválido — usa 0=Speed 1=Stamina 2=Power 3=Guts 4=Wit")]
    InvalidStat,

    #[msg("No puedes correr todavía, quedan turnos de entrenamiento")]
    CannotRaceYet,
}
