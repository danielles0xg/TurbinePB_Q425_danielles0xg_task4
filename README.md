# Lending market


Modular peer-to-peer lending protocol, where borrowers and Lenders choose their risk/reward (LTV/APR), is perpetual: no expirations, and no time-based liquidations, is an orderbook: users may determine yield/interest individually by no means of trade intermediaries.

Note: this project took task3 project as the starter point

## User stories

#### Fees

- Protocol charges 1% fee of loan amount to borrower when taking a loan offer
- Protocol charges 2% fee of loan repay amount to lender
loan is repaid

#### For Lenders
- As a lender I am able to deposit capital to place a lending offer
- As a lender I am able to set the interest rate and LTV of my offer
- As a lender I can decide when to request for repayment with 48hrs notice


#### For Borrowers
- As a borrower I can borrow capital against my collateral token
- As a borrower I repay the Loan and get back my collateral token


## PDA Architecture 
(inspired by builders capstone youtube presentations, Im adding the pda architecture)

 1. LendingMarket
    - Seeds: `["lending_market"]`
    - Authority: Admin
    - Purpose: Global protocol configuration and fee settings

 2. AssetPairMarket
    - Seeds: `["asset_pair", loan_mint.key(), collateral_mint.key()]`
    - Authority: Admin (via LendingMarket)
    - Purpose: Defines which loan/collateral token pairs are allowed for trading


 3. LendingOffer
    - Seeds: `["lending_offer", lender.key(), offer_id.to_le_bytes()]`
    - Authority: Lender
    - Purpose: Individual lender's offer with custom terms (orderbook entry)

 4. Escrow
    - Seeds: `["escrow", lending_offer.key()]`
    - Authority: Program PDA
    - Purpose: Holds lender's capital when offer is created
    - Token Account: Owned by Escrow PDA, holds loan tokens

 5. Loan
    - Seeds: `["loan", lending_offer.key(), borrower.key()]`
    - Authority: Borrower and Lender (joint)
    - Purpose: Tracks active loan with interest accrual

 6. CollateralVault
    - Seeds: `["collateral", loan.key()]`
    - Authority: Program PDA
    - Purpose: Holds borrower's collateral during active loan
    - Token Account: Owned by CollateralVault PDA, holds collateral tokens



## License

- MIT