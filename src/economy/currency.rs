use core::ops::{Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign};
use core::fmt::{Debug, Display, Formatter, Result as FmtResult};

/// A unit of currency representing in-game value.
/// This is used to purchase units and extra moves.
/// This is a signed integer, so it can represent debt.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Currency {
    /// The amount of currency.
    amount: i32,
}

impl Currency {
    /// A standard unit of currency.
    pub fn doubloon() -> Self {
        Self::new(10)
    }

    /// A standard subunit of currency.
    /// This is the smallest unit of currency.
    pub fn penny() -> Self {
        Self::new(1)
    }

    /// A zero amount of currency.
    pub fn zero() -> Self {
        Self::new(0)
    }

    /// Create a new currency with the given amount.
    #[inline]
    fn new(amount: i32) -> Self {
        Self { amount }
    }

    /// Is the currency amount zero?
    pub fn is_zero(&self) -> bool {
        self.amount == 0
    }

    /// Is the currency amount negative?
    pub fn is_debt(&self) -> bool {
        self.amount < 0
    }

    /// Is the currency amount positive?
    pub fn is_surplus(&self) -> bool {
        self.amount > 0
    }

    pub fn get_amount(&self) -> i32 {
        self.amount
    }
}

impl Default for Currency {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let required_width = if self.is_debt() {
            1 + self.amount.abs().max(1).ilog10()
        } else {
            self.amount.max(1).ilog10()
        } as usize;
        if let Some(mut width) = f.width() {
            while width > required_width {
                write!(f, " ")?;
                width -= 1;
            }
        }

        if self.is_debt() {
            write!(f, "-¢{}", self.amount)
        } else {
            write!(f, "¢{}", self.amount)
        }
    }
}

impl Debug for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self)
    }
}

impl Add for Currency {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.amount + rhs.amount)
    }
}

impl Sub for Currency {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.amount - rhs.amount)
    }
}

impl Mul<i32> for Currency {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self::new(self.amount * rhs)
    }
}

impl Mul<u32> for Currency {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self::new(self.amount * rhs as i32)
    }
}

impl Mul<f64> for Currency {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new((self.amount as f64 * rhs) as i32)
    }
}

impl Div<i32> for Currency {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self::new(self.amount / rhs)
    }
}

impl Div<u32> for Currency {
    type Output = Self;

    fn div(self, rhs: u32) -> Self::Output {
        Self::new(self.amount / rhs as i32)
    }
}

impl Div<f64> for Currency {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self::new((self.amount as f64 / rhs) as i32)
    }
}

impl AddAssign for Currency {
    fn add_assign(&mut self, rhs: Self) {
        self.amount += rhs.amount;
    }
}

impl SubAssign for Currency {
    fn sub_assign(&mut self, rhs: Self) {
        self.amount -= rhs.amount;
    }
}

impl MulAssign<i32> for Currency {
    fn mul_assign(&mut self, rhs: i32) {
        self.amount *= rhs;
    }
}

impl DivAssign<i32> for Currency {
    fn div_assign(&mut self, rhs: i32) {
        self.amount /= rhs;
    }
}

impl Div for Currency {
    type Output = f64;

    fn div(self, rhs: Self) -> Self::Output {
        self.amount as f64 / rhs.amount as f64
    }
}