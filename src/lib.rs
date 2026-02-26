use anchor_lang::prelude::*;

declare_id!("");

#[program]
pub mod library {
    use super::*;

    pub fn create_library() -> Result<()> {

    }
}

#[account]
#[derive(InitSpace)]
pub struct Library {
    owner: Pubkey,

    #[max_len(80)]
    name: String,

    #[max_len(10)]
    books: Vec<Book>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq, Debug)]
pub struct Book {
    #[max_len(80)]
    title: String,

    pages: u16,

    available: bool,
}

#[derive(Accounts)]
pub struct NewLibrary {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = Library::INIT_SPACE + 8,
        seed = [b"library", owner.key().as_ref()],
        bump
    )]
    pub library: Account<'info, Library>,
    
    pub system_program: Program<'info, System>,
}

pub struct NewBook {
    pub owner: Signer<'info>,
    
    #[account(mut)]
    pub library: Account<'info, Library>,
}
