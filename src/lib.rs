// VERSION: 6
// Date: 4 October 2024
// Change:
// - Remove price feed as it was causing issues with the program
// - Add a new function to allow users to claim their EVM addresses
// - Security text

// Import necessary modules from the Anchor framework and the standard library.
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{ program::invoke, system_instruction };
use solana_security_txt::security_txt;

// Declare the unique identifier for this Solana program.
declare_id!("AquaFurRSVeVin1wJPmf7bvP6fCEKBqQbdpq6fr3aPy5");

// Define the main program module.
#[program]
pub mod presale_program {
    // Import all symbols from the outer scope (to use without full path).
    use super::*;

    // Function to initialize a new Presale account.
    pub fn initialize(ctx: Context<Initialize>, payment_wallet: Pubkey, rate: u64) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        // Set the owner of the presale to the account initializing it.
        presale.owner = *ctx.accounts.owner.key;

        // Set the initial token rate for the presale.
        presale.rate = rate;

        // Set the initial payment wallet
        presale.payment_wallet = payment_wallet;

        // Ensure the presale starts in an active state (not paused).
        presale.is_paused = false;

        Ok(())
    }

    // Function to allow users to buy tokens during the presale.
    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
        // Log this value into the transaction log
        msg!("StakeLog: Buyer: {}", *ctx.accounts.buyer.key);
        msg!("StakeLog: Amount: {}", amount);
        Ok(())
    }

    // Function for users to submit their EVM addresses.
    pub fn claim_evm(ctx: Context<ClaimEVM>, evm_address: String) -> Result<()> {
        // Log the user's public key and EVM address.
        msg!("ClaimEVMLog: User: {}", *ctx.accounts.user.key);
        msg!("ClaimEVMLog: EVM Address: {}", evm_address);
        Ok(())
    }

    // Function to allow users to buy tokens during the presale.
    pub fn buy_tokens(
        ctx: Context<BuyTokens>,
        sol_amount: u64,
        stake: bool,
        evm_address: String
    ) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        // Ensure the presale is not paused before proceeding.
        require!(!presale.is_paused, ErrorCode::PresaleIsPaused);

        // Ensure that the payment wallet provides is the correct one.
        require_keys_eq!(
            presale.payment_wallet,
            ctx.accounts.payment_wallet.key(),
            ErrorCode::InvalidPaymentWallet
        );

        // Perform the SOL transfer
        let sender = &ctx.accounts.buyer.to_account_info();
        let receiver = &ctx.accounts.payment_wallet.to_account_info();

        // Ensure the sender's account is not the same as the receiver's
        if sender.key() == receiver.key() {
            return Err(ProgramError::InvalidArgument.into());
        }

        // Construct the transfer instruction to the payment wallet
        let transfer_instruction = system_instruction::transfer(
            sender.key,
            receiver.key,
            sol_amount
        );

        // Invoke the transfer instruction
        invoke(
            &transfer_instruction,
            &[
                sender.to_account_info(),
                receiver.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ]
        )?;

        // Log this value into the transaction log
        msg!("BuyerLog: Buyer: {}", *ctx.accounts.buyer.key);
        msg!("BuyerLog: SOL amount: {}", sol_amount);
        msg!("BuyerLog: Price: ~ {}", presale.rate);
        msg!("BuyerLog: Stake: ~ {}", stake);
        msg!("BuyerLog: EVM Address: {}", evm_address);

        Ok(())
    }

    // Function to withdraw SOL from the presale account.
    pub fn withdraw_sol(ctx: Context<WithdrawSol>, amount: u64) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        // Ensure that the caller is the owner of the presale.
        require_keys_eq!(presale.owner, ctx.accounts.owner.key(), ErrorCode::Unauthorized);

        // Deduct the specified amount of SOL from the presale account.
        **presale.to_account_info().try_borrow_mut_lamports()? -= amount;

        // Add the specified amount of SOL to the recipient's account.
        **ctx.accounts.recipient.try_borrow_mut_lamports()? += amount;

        Ok(())
    }

    // Function to change the rate of tokens per SOL.
    pub fn change_rate(ctx: Context<ChangeRate>, new_rate: u64) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        // Ensure that the caller is the owner of the presale.
        require_keys_eq!(presale.owner, ctx.accounts.owner.key(), ErrorCode::Unauthorized);

        // Update the rate at which tokens are sold.
        presale.rate = new_rate;

        Ok(())
    }

    // Function to change the payment wallet.
    pub fn change_payment_wallet(
        ctx: Context<ChangePaymentWallet>,
        new_wallet: Pubkey
    ) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        // Ensure that the caller is the owner of the presale.
        require_keys_eq!(presale.owner, ctx.accounts.owner.key(), ErrorCode::Unauthorized);

        // Update the rate at which tokens are sold.
        presale.payment_wallet = new_wallet;

        Ok(())
    }

    // Function to pause or resume the presale.
    pub fn pause_presale(ctx: Context<PausePresale>, pause: bool) -> Result<()> {
        let presale = &mut ctx.accounts.presale;

        // Ensure that the caller is the owner of the presale.
        require_keys_eq!(presale.owner, ctx.accounts.owner.key(), ErrorCode::Unauthorized);

        // Set the presale's paused state according to the function call.
        presale.is_paused = pause;

        Ok(())
    }
}

// Account structs used in different transactions.

#[derive(Accounts)]
pub struct Initialize<'info> {
    // Define the presale account that will be created and owned by the caller.
    #[account(init, payer = owner, space = 500)]
    pub presale: Account<'info, Presale>,

    // The account paying for the transaction and owning the new presale account.
    #[account(mut)]
    pub owner: Signer<'info>,

    // Reference to the system program, used for creating accounts.
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    // The presale account from which tokens are being bought.
    #[account(mut)]
    pub presale: Account<'info, Presale>,

    // The buyer of the tokens.
    // The #[account(mut, signer)] attribute on sender ensures that the account is both mutable (to deduct SOL)
    // and a signer of the transaction (implying that the caller of this function must be the sender).
    #[account(mut, signer)]
    pub buyer: Signer<'info>,

    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(mut)]
    pub payment_wallet: AccountInfo<'info>,

    // Add the system program account to facilitate the transfer of SOL
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    // The presale account
    #[account(mut)]
    pub presale: Account<'info, Presale>,

    // The buyer of the tokens.
    // The #[account(mut, signer)] attribute on sender ensures that the account is both mutable (to deduct SOL)
    // and a signer of the transaction (implying that the caller of this function must be the sender).
    #[account(mut, signer)]
    pub buyer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimEVM<'info> {
    // The presale account
    #[account(mut)]
    pub presale: Account<'info, Presale>,

    // The user submitting their EVM address.
    // The #[account(mut, signer)] attribute on sender ensures that the account is both mutable (to deduct SOL)
    // and a signer of the transaction (implying that the caller of this function must be the sender).
    #[account(mut, signer)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawSol<'info> {
    // The presale account from which SOL will be withdrawn.
    #[account(mut)]
    pub presale: Account<'info, Presale>,

    // The recipient account to which SOL will be sent.
    #[account(mut)]
    pub recipient: Signer<'info>,

    // The owner of the presale account who is authorized to perform withdrawals.
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct ChangeRate<'info> {
    // The presale account for which the token sale rate will be changed.
    #[account(mut)]
    pub presale: Account<'info, Presale>,

    // The owner of the presale account, authorized to change the rate.
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct ChangePaymentWallet<'info> {
    // The presale account for which the payment wallet will be changed.
    #[account(mut)]
    pub presale: Account<'info, Presale>,

    // The owner of the presale account, authorized to change the payment wallet.
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct PausePresale<'info> {
    // The presale account that will be paused or resumed.
    #[account(mut)]
    pub presale: Account<'info, Presale>,

    // The owner of the presale account, authorized to pause or resume it.
    pub owner: Signer<'info>,
}

// The main Presale account structure.
#[account]
pub struct Presale {
    // The public key of the owner of the presale.
    pub owner: Pubkey,

    // The rate of tokens per SOL.
    pub rate: u64,

    // The wallet for sending the SOL payments to
    pub payment_wallet: Pubkey,

    // Flag indicating whether the presale is paused.
    pub is_paused: bool,
}

// Custom error codes used in the program.
#[error_code]
pub enum ErrorCode {
    // Indicates that the presale is currently paused.
    #[msg("The presale is currently paused.")]
    PresaleIsPaused,

    // Indicates an overflow error, likely during token allocation calculation.
    #[msg("Operation overflowed.")]
    Overflow,

    // Indicates an underflow error, likely during token allocation calculation.
    #[msg("Operation underflowed.")]
    Underflow,

    // Indicates an unauthorized attempt to perform an operation.
    #[msg("Unauthorized.")]
    Unauthorized,

    // Indicates an unauthorized attempt to perform an operation.
    #[msg("Invalid payment wallet provided.")]
    InvalidPaymentWallet,

    // Indicates that the amount of SOL transferred does not match the expected amount.
    #[msg("Invalid amount of SOL transferred.")]
    InvalidAmountTransferred,
}

security_txt! {
    // Required fields
    name: "Aquadoge Presale",
    project_url: "https://aquadoge.com",
    contacts: "email:team@aquadoge.com,link:https://aquadoge.com/security,telegram:flipky386343",
    policy: "https://github.com/teamaquadoge/presale-solana/blob/master/SECURITY.md",

    // Optional Fields
    preferred_languages: "en",
    source_code: "https://github.com/teamaquadoge/presale-solana",
    acknowledgements: "Thanks for finding a bug in our program! Please report it to team@aquadoge.com"
}
