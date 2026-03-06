use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, TokenAccount, Transfer, MintTo};

declare_id!("TestCoin111111111111111111111111111111");

#[program]
pub mod test_coin {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, buy_tax: u64, sell_tax: u64) -> Result<()> {
        ctx.accounts.mint_data.buy_tax = buy_tax;
        ctx.accounts.mint_data.sell_tax = sell_tax;
        ctx.accounts.mint_data.authority = ctx.accounts.authority.key();
        ctx.accounts.mint_data.mint_authority = ctx.accounts.mint.key();
        
        let amount = 21000 * 10_000_000;
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.tax_vault.to_account_info(),
                    authority: ctx.accounts.mint.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }

    pub fn set_tax_rate(ctx: Context<SetTaxRate>, buy_tax: u64, sell_tax: u64) -> Result<()> {
        ctx.accounts.mint_data.buy_tax = buy_tax;
        ctx.accounts.mint_data.sell_tax = sell_tax;
        Ok(())
    }

    pub fn transfer_with_tax(ctx: Context<TransferTax>, amount: u64, is_sell: bool) -> Result<()> {
        let tax_rate = if is_sell { ctx.accounts.mint_data.sell_tax } else { ctx.accounts.mint_data.buy_tax };
        let tax_amount = amount * tax_rate / 10000;
        let transfer_amount = amount - tax_amount;

        if transfer_amount > 0 {
            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer { from: ctx.accounts.from.to_account_info(), to: ctx.accounts.to.to_account_info(), authority: ctx.accounts.from.to_account_info() },
                ),
                transfer_amount,
            )?;
        }
        if tax_amount > 0 {
            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer { from: ctx.accounts.from.to_account_info(), to: ctx.accounts.tax_vault.to_account_info(), authority: ctx.accounts.from.to_account_info() },
                ),
                tax_amount,
            )?;
        }
        Ok(())
    }

    pub fn withdraw_tax(ctx: Context<WithdrawTax>, amount: u64) -> Result<()> {
        let vault_balance = ctx.accounts.tax_vault.amount;
        require!(amount <= vault_balance, ErrorCode::InsufficientBalance);
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer { from: ctx.accounts.tax_vault.to_account_info(), to: ctx.accounts.destination.to_account_info(), authority: ctx.accounts.mint.to_account_info() },
            ),
            amount,
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, mint::decimals = 9, mint::authority = mint, seeds = [b"mint"], bump)]
    pub mint: Account<'info, Mint>,
    #[account(init, payer = authority, seeds = [b"tax_vault"], bump, token::mint = mint, token::authority = mint)]
    pub tax_vault: Account<'info, TokenAccount>,
    #[account(init, payer = authority, space = 8 + 64, seeds = [b"mint_data"], bump)]
    pub mint_data: Account<'info, MintData>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SetTaxRate<'info> {
    #[account(seeds = [b"mint_data"], bump)]
    pub mint_data: Account<'info, MintData>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct TransferTax<'info> {
    #[account(seeds = [b"mint_data"], bump)]
    pub mint_data: Account<'info, MintData>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    #[account(mut)]
    pub tax_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawTax<'info> {
    #[account(seeds = [b"mint"], bump)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub tax_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct MintData {
    pub buy_tax: u64,
    pub sell_tax: u64,
    pub authority: Pubkey,
    pub mint_authority: Pubkey,
}

#[error_code]
pub enum ErrorCode {
    TaxRateTooHigh,
    InsufficientBalance,
}
