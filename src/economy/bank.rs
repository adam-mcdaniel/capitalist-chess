use super::{Currency, Color, Market, Move, Board, Sector};
use log::{info, debug, error};
use core::fmt::{Display, Formatter, Result as FmtResult};

/// Federal bank for each player.
/// This adds an economic element to the game. Each player has a bank
/// which gains income depending on their territory and can be used to
/// purchase units, and extra moves.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Bank {
    /// The color of the bank.
    color: Color,
    /// The balance of the bank.
    balance: Currency,
    /// The market for the bank.
    market: Market,

    /// The sectors owned by the bank.
    sectors: [bool; Sector::NUM_SECTORS]
}

impl Bank {
    /// Create a new bank.
    /// This will initialize the bank with the given color and market.
    pub fn new(color: Color, market: Market) -> Self {
        let mut sectors = [false; Sector::NUM_SECTORS];
        match color {
            Color::White => {
                sectors[0] = true;
                sectors[1] = true;
                sectors[2] = true;
                sectors[3] = true;
            },
            Color::Black => {
                sectors[12] = true;
                sectors[13] = true;
                sectors[14] = true;
                sectors[15] = true;
            },
        }

        Self {
            color,
            balance: Currency::zero(),
            market,
            sectors,
        }
    }

    /// Can this bank afford the given move?
    /// This will check if the bank has enough money to purchase the given move.
    #[inline]
    pub fn can_afford(&self, player_move: &Move) -> bool {
        self.balance >= self.market.get_move_value(player_move)
    }

    /// Add money to the bank.
    /// This will add the given amount of money to the bank's balance.
    #[inline]
    pub fn deposit(&mut self, amount: Currency) {
        self.balance += amount;
    }

    /// Withdraw money from the bank.
    /// This will subtract the given amount of money from the bank's balance.
    /// If the bank does not have enough money, this will return an error.
    pub fn withdraw(&mut self, amount: Currency) -> Result<(), ()> {
        if self.balance < amount {
            error!("Bank for {:?} does not have enough money to withdraw {:?}", self.get_color(), amount);
            return Err(());
        }
        self.balance -= amount;
        Ok(())
    }

    /// Purchase a move from the bank.
    /// This will subtract the cost of the move from the bank's balance.
    /// If the bank does not have enough money, this will return an error.
    pub fn purchase(&mut self, player_move: &Move) -> Result<(), ()> {
        info!("Bank for {:?} purchasing move {player_move:?}", self.get_color());
        self.withdraw(self.market.get_move_value(player_move))
    }

    /// Get the color of the bank.
    #[inline]
    pub fn get_color(&self) -> Color {
        self.color
    }

    /// Get the balance of the bank.
    #[inline]
    pub fn get_balance(&self) -> Currency {
        self.balance
    }

    /// Get the market of the bank.
    #[inline]
    pub fn get_market(&self) -> Market {
        self.market
    }

    /// Take a census of the board.
    /// This will check which sectors are controlled by the bank,
    /// and update the bank's income.
    pub fn perform_census(&mut self, board: &Board) {
        info!("Taking census for bank controlled by {:?}", self.get_color());
        // Count the board's sectors
        self.sectors = board.get_controlled_sectors(self.get_color());

        // Update the bank's balance
        self.balance += self.calculate_income();
    }

    /// Calculate income based on the sectors controlled by the bank.
    fn calculate_income(&self) -> Currency {
        let mut income = Currency::zero();
        for (i, sector) in self.sectors.iter().enumerate() {
            if !sector {
                continue;
            }
            let income_for_sector = self.get_market().get_sector_value(Sector::from_index(i));
            debug!("Sector {:?} is controlled by {:?} and is worth {:?}", Sector::from_index(i), self.get_color(), income_for_sector);
            income += income_for_sector;
        }
        income
    }
}

impl Display for Bank {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "╔═══════════════╗\n")?;
        write!(f, "║ {:5?} {:5} ║\n", self.get_color(), self.balance)?;
        write!(f, "╚═══════════════╝\n")
    }
}