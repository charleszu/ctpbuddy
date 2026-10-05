//! Fixed-point currency (1e-6 CNY per unit).
//!
//! Every booked amount (balance, margin, commission, PnL) is an integer, so
//! accumulation is exact and independent of summation order — the ledger
//! iterates `HashMap`s, and `f64` sums over a randomly-ordered map are not
//! reproducible across processes. Amounts derived from prices and rates are
//! computed in `f64` once and rounded half-away-from-zero on entry
//! ([`Money::from_f64`]); prices and rates themselves stay `f64`.

use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

const SCALE: f64 = 1_000_000.0;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Money(i64);

impl Money {
    pub const ZERO: Money = Money(0);

    /// Round to the nearest unit; non-finite input becomes zero, huge input
    /// saturates.
    pub fn from_f64(x: f64) -> Money {
        if !x.is_finite() {
            return Money(0);
        }
        Money((x * SCALE).round() as i64)
    }

    pub fn to_f64(self) -> f64 {
        self.0 as f64 / SCALE
    }

    pub fn from_units(u: i64) -> Money {
        Money(u)
    }

    /// Raw units (1e-6 CNY): exact, used for snapshots and tests.
    pub fn units(self) -> i64 {
        self.0
    }

    pub fn is_positive(self) -> bool {
        self.0 > 0
    }

    /// `self * num / den` rounded half away from zero, via 128-bit
    /// intermediates (pro-rata releases). `den <= 0` yields zero.
    pub fn ratio(self, num: i64, den: i64) -> Money {
        if den <= 0 {
            return Money(0);
        }
        let n = self.0 as i128 * num as i128;
        let d = den as i128;
        let q = if n >= 0 {
            (n + d / 2) / d
        } else {
            (n - d / 2) / d
        };
        Money(q.clamp(i64::MIN as i128, i64::MAX as i128) as i64)
    }

    /// Clamp negatives to zero.
    pub fn floor_zero(self) -> Money {
        Money(self.0.max(0))
    }
}

impl fmt::Debug for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_f64())
    }
}

impl Add for Money {
    type Output = Money;
    fn add(self, o: Money) -> Money {
        Money(self.0.saturating_add(o.0))
    }
}

impl Sub for Money {
    type Output = Money;
    fn sub(self, o: Money) -> Money {
        Money(self.0.saturating_sub(o.0))
    }
}

impl Neg for Money {
    type Output = Money;
    fn neg(self) -> Money {
        Money(self.0.saturating_neg())
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, o: Money) {
        *self = *self + o;
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, o: Money) {
        *self = *self - o;
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Money {
        iter.fold(Money::ZERO, |a, b| a + b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounding_and_roundtrip() {
        assert_eq!(Money::from_f64(0.1).to_f64(), 0.1);
        assert_eq!(Money::from_f64(2_000_000.0).units(), 2_000_000_000_000);
        assert_eq!(Money::from_f64(-0.0000005).units(), -1);
        assert_eq!(Money::from_f64(f64::NAN), Money::ZERO);
        assert_eq!(Money::from_f64(f64::INFINITY), Money::ZERO);
    }

    #[test]
    fn accumulation_is_exact_and_order_independent() {
        let xs: Vec<Money> = (0..1000).map(|_| Money::from_f64(0.1)).collect();
        let fwd: Money = xs.iter().copied().sum();
        let rev: Money = xs.iter().rev().copied().sum();
        assert_eq!(fwd, rev);
        assert_eq!(fwd.to_f64(), 100.0);
        // the f64 equivalent drifts
        let f: f64 = (0..1000).map(|_| 0.1f64).sum();
        assert_ne!(f, 100.0);
    }

    #[test]
    fn pro_rata_release_sums_to_original() {
        let m = Money::from_f64(100.0);
        let a = m.ratio(1, 3);
        let b = m.ratio(2, 3);
        assert_eq!((a + b).units(), m.units());
        assert_eq!(m.ratio(5, 0), Money::ZERO);
    }
}
