//! NA propagation operations
//!
//! This module provides ALL arithmetic and comparison operations for Pine Script.
//! **NEVER** implement NA checks inline - always use functions from this module.
//!
//! # Pine Script NA Semantics
//!
//! - `na + 1 == na` (not 1)
//! - `na * 0 == na` (not 0)
//! - `na == na == false` (like IEEE 754 NaN)
//! - `na != na == true`
//! - `nz(na, 0) == 0`

use crate::value::Value;

//===========================================================================
// Arithmetic Operations
//===========================================================================

/// Add two values with NA propagation
///
/// # Examples
/// ```
/// use pine_runtime::value::Value;
/// use pine_runtime::na_ops::add;
///
/// assert_eq!(add(&Value::Int(2), &Value::Int(3)), Value::Int(5));
/// assert!(add(&Value::Na, &Value::Int(1)).is_na());
/// ```
pub fn add(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        return Value::Na;
    }
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
        (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 + b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a + *b as f64),
        _ => Value::Na,
    }
}

/// Subtract two values with NA propagation
pub fn sub(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        return Value::Na;
    }
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
        (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 - b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a - *b as f64),
        _ => Value::Na,
    }
}

/// Multiply two values with NA propagation
pub fn mul(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        return Value::Na;
    }
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
        (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 * b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a * *b as f64),
        _ => Value::Na,
    }
}

/// Divide two values with NA propagation
///
/// Division by zero returns NA (not infinity).
pub fn div(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        return Value::Na;
    }
    match (lhs, rhs) {
        (_, Value::Int(0)) => Value::Na,
        (_, Value::Float(b)) if *b == 0.0 => Value::Na,
        (Value::Int(a), Value::Int(b)) => Value::Float(*a as f64 / *b as f64),
        (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 / b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a / *b as f64),
        _ => Value::Na,
    }
}

/// Integer division with NA propagation
///
/// Returns the integer part of the division, truncating toward zero.
pub fn idiv(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        return Value::Na;
    }
    match (lhs, rhs) {
        (_, Value::Int(0)) => Value::Na,
        (_, Value::Float(b)) if *b == 0.0 => Value::Na,
        (Value::Int(a), Value::Int(b)) => Value::Int(a / b),
        (Value::Float(a), Value::Float(b)) => Value::Int((*a / *b) as i64),
        (Value::Int(a), Value::Float(b)) => Value::Int((*a as f64 / b) as i64),
        (Value::Float(a), Value::Int(b)) => Value::Int((a / *b as f64) as i64),
        _ => Value::Na,
    }
}

/// Modulo (remainder) with NA propagation
pub fn modulo(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        return Value::Na;
    }
    match (lhs, rhs) {
        (_, Value::Int(0)) => Value::Na,
        (_, Value::Float(b)) if *b == 0.0 => Value::Na,
        (Value::Int(a), Value::Int(b)) => Value::Int(a % b),
        (Value::Float(a), Value::Float(b)) => Value::Float(a % b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 % b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a % *b as f64),
        _ => Value::Na,
    }
}

/// Negate a value with NA propagation
pub fn neg(val: &Value) -> Value {
    if val.is_na() {
        return Value::Na;
    }
    match val {
        Value::Int(i) => Value::Int(-i),
        Value::Float(f) => Value::Float(-f),
        _ => Value::Na,
    }
}

/// Absolute value with NA propagation
pub fn abs(val: &Value) -> Value {
    if val.is_na() {
        return Value::Na;
    }
    match val {
        Value::Int(i) => Value::Int(i.abs()),
        Value::Float(f) => Value::Float(f.abs()),
        _ => Value::Na,
    }
}

//===========================================================================
// Comparison Operations
//===========================================================================

/// Compare two values for equality
///
/// **IMPORTANT**: In Pine Script, `na == na` returns `false` (like IEEE 754 NaN).
pub fn eq(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        // na == na is false, na == x is false
        Value::Bool(false)
    } else {
        Value::Bool(lhs == rhs)
    }
}

/// Compare two values for inequality
///
/// **IMPORTANT**: In Pine Script, `na != na` returns `true` (like IEEE 754 NaN).
pub fn ne(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        // na != na is true, na != x is true
        Value::Bool(true)
    } else {
        Value::Bool(lhs != rhs)
    }
}

/// Less than comparison with NA propagation
///
/// Any comparison with NA returns `false`.
pub fn lt(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        Value::Bool(false)
    } else {
        match (lhs.as_float(), rhs.as_float()) {
            (Some(a), Some(b)) => Value::Bool(a < b),
            _ => Value::Na,
        }
    }
}

/// Less than or equal comparison with NA propagation
pub fn le(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        Value::Bool(false)
    } else {
        match (lhs.as_float(), rhs.as_float()) {
            (Some(a), Some(b)) => Value::Bool(a <= b),
            _ => Value::Na,
        }
    }
}

/// Greater than comparison with NA propagation
pub fn gt(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        Value::Bool(false)
    } else {
        match (lhs.as_float(), rhs.as_float()) {
            (Some(a), Some(b)) => Value::Bool(a > b),
            _ => Value::Na,
        }
    }
}

/// Greater than or equal comparison with NA propagation
pub fn ge(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        Value::Bool(false)
    } else {
        match (lhs.as_float(), rhs.as_float()) {
            (Some(a), Some(b)) => Value::Bool(a >= b),
            _ => Value::Na,
        }
    }
}

//===========================================================================
// Logical Operations
//===========================================================================

/// Logical AND with NA propagation
///
/// - `true and na` -> `na`
/// - `false and na` -> `false`
/// - `na and true` -> `na`
/// - `na and false` -> `na`
pub fn and(lhs: &Value, rhs: &Value) -> Value {
    match (lhs.as_bool(), rhs.as_bool()) {
        (Some(a), Some(b)) => Value::Bool(a && b),
        (Some(false), _) => Value::Bool(false), // false and x = false
        (_, Some(false)) => Value::Bool(false), // x and false = false
        _ => Value::Na,                         // anything with na = na
    }
}

/// Logical OR with NA propagation
///
/// - `true or na` -> `true`
/// - `false or na` -> `na`
/// - `na or true` -> `true`
/// - `na or false` -> `na`
pub fn or(lhs: &Value, rhs: &Value) -> Value {
    match (lhs.as_bool(), rhs.as_bool()) {
        (Some(a), Some(b)) => Value::Bool(a || b),
        (Some(true), _) => Value::Bool(true), // true or x = true
        (_, Some(true)) => Value::Bool(true), // x or true = true
        _ => Value::Na,                       // anything with na = na
    }
}

/// Logical NOT with NA propagation
pub fn not(val: &Value) -> Value {
    if val.is_na() {
        return Value::Na;
    }
    match val.as_bool() {
        Some(b) => Value::Bool(!b),
        None => Value::Na,
    }
}

//===========================================================================
// Bitwise Operations
//===========================================================================

/// Bitwise AND with NA propagation
pub fn bit_and(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        return Value::Na;
    }
    match (lhs.as_int(), rhs.as_int()) {
        (Some(a), Some(b)) => Value::Int(a & b),
        _ => Value::Na,
    }
}

/// Bitwise OR with NA propagation
pub fn bit_or(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        return Value::Na;
    }
    match (lhs.as_int(), rhs.as_int()) {
        (Some(a), Some(b)) => Value::Int(a | b),
        _ => Value::Na,
    }
}

/// Bitwise XOR with NA propagation
pub fn bit_xor(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        return Value::Na;
    }
    match (lhs.as_int(), rhs.as_int()) {
        (Some(a), Some(b)) => Value::Int(a ^ b),
        _ => Value::Na,
    }
}

/// Bitwise NOT with NA propagation
pub fn bit_not(val: &Value) -> Value {
    if val.is_na() {
        return Value::Na;
    }
    match val.as_int() {
        Some(i) => Value::Int(!i),
        None => Value::Na,
    }
}

/// Left shift with NA propagation
pub fn shl(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        return Value::Na;
    }
    match (lhs.as_int(), rhs.as_int()) {
        (Some(a), Some(b)) => Value::Int(a << b),
        _ => Value::Na,
    }
}

/// Right shift with NA propagation
pub fn shr(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() || rhs.is_na() {
        return Value::Na;
    }
    match (lhs.as_int(), rhs.as_int()) {
        (Some(a), Some(b)) => Value::Int(a >> b),
        _ => Value::Na,
    }
}

//===========================================================================
// NA Handling Functions
//===========================================================================

/// Check if a value is NA
pub fn is_na(value: &Value) -> bool {
    value.is_na()
}

/// Replace NA with a default value (nz = "not zero" but actually means "not na")
///
/// # Examples
/// ```
/// use pine_runtime::value::Value;
/// use pine_runtime::na_ops::nz;
///
/// assert_eq!(nz(&Value::Na, &Value::Int(0)), Value::Int(0));
/// assert_eq!(nz(&Value::Int(5), &Value::Int(0)), Value::Int(5));
/// ```
pub fn nz(value: &Value, default: &Value) -> Value {
    if value.is_na() {
        default.clone()
    } else {
        value.clone()
    }
}

/// Conditional operator (if cond then a else b) with NA propagation
///
/// If the condition is NA, returns NA.
pub fn if_then_else(cond: &Value, then_val: &Value, else_val: &Value) -> Value {
    if cond.is_na() {
        Value::Na
    } else if cond.is_truthy() {
        then_val.clone()
    } else {
        else_val.clone()
    }
}

/// Coalesce operator (??) - returns first non-NA value
///
/// # Examples
/// ```
/// use pine_runtime::value::Value;
/// use pine_runtime::na_ops::coalesce;
///
/// assert_eq!(coalesce(&Value::Na, &Value::Int(1)), Value::Int(1));
/// assert_eq!(coalesce(&Value::Int(2), &Value::Int(1)), Value::Int(2));
/// ```
pub fn coalesce(lhs: &Value, rhs: &Value) -> Value {
    if lhs.is_na() {
        rhs.clone()
    } else {
        lhs.clone()
    }
}

//===========================================================================
// Math Operations
//===========================================================================

/// Power/exponentiation with NA propagation
pub fn pow(base: &Value, exp: &Value) -> Value {
    if base.is_na() || exp.is_na() {
        return Value::Na;
    }
    match (base.as_float(), exp.as_float()) {
        (Some(b), Some(e)) => {
            let result = b.powf(e);
            if result.is_finite() {
                Value::Float(result)
            } else {
                Value::Na
            }
        }
        _ => Value::Na,
    }
}

/// Square root with NA propagation
pub fn sqrt(val: &Value) -> Value {
    if val.is_na() {
        return Value::Na;
    }
    match val.as_float() {
        Some(v) if v >= 0.0 => Value::Float(v.sqrt()),
        _ => Value::Na,
    }
}

/// Natural logarithm with NA propagation
pub fn ln(val: &Value) -> Value {
    if val.is_na() {
        return Value::Na;
    }
    match val.as_float() {
        Some(v) if v > 0.0 => Value::Float(v.ln()),
        _ => Value::Na,
    }
}

/// Base-10 logarithm with NA propagation
pub fn log10(val: &Value) -> Value {
    if val.is_na() {
        return Value::Na;
    }
    match val.as_float() {
        Some(v) if v > 0.0 => Value::Float(v.log10()),
        _ => Value::Na,
    }
}

/// Exponential (e^x) with NA propagation
pub fn exp(val: &Value) -> Value {
    if val.is_na() {
        return Value::Na;
    }
    match val.as_float() {
        Some(v) => {
            let result = v.exp();
            if result.is_finite() {
                Value::Float(result)
            } else {
                Value::Na
            }
        }
        None => Value::Na,
    }
}

//===========================================================================
// Min/Max Operations
//===========================================================================

/// Minimum of two values with NA propagation
pub fn min(a: &Value, b: &Value) -> Value {
    if a.is_na() || b.is_na() {
        return Value::Na;
    }
    match (a.as_float(), b.as_float()) {
        (Some(x), Some(y)) => {
            if x <= y {
                a.clone()
            } else {
                b.clone()
            }
        }
        _ => Value::Na,
    }
}

/// Maximum of two values with NA propagation
pub fn max(a: &Value, b: &Value) -> Value {
    if a.is_na() || b.is_na() {
        return Value::Na;
    }
    match (a.as_float(), b.as_float()) {
        (Some(x), Some(y)) => {
            if x >= y {
                a.clone()
            } else {
                b.clone()
            }
        }
        _ => Value::Na,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic() {
        // Addition
        assert_eq!(add(&Value::Int(2), &Value::Int(3)), Value::Int(5));
        assert_eq!(
            add(&Value::Float(2.5), &Value::Float(3.5)),
            Value::Float(6.0)
        );
        assert_eq!(add(&Value::Int(2), &Value::Float(3.5)), Value::Float(5.5));

        // NA propagation
        assert!(add(&Value::Na, &Value::Int(1)).is_na());
        assert!(add(&Value::Int(1), &Value::Na).is_na());

        // Multiplication - na * 0 = na (not 0!)
        assert!(mul(&Value::Na, &Value::Int(0)).is_na());

        // Division by zero = na
        assert!(div(&Value::Int(1), &Value::Int(0)).is_na());

        // Integer division
        assert_eq!(idiv(&Value::Int(7), &Value::Int(2)), Value::Int(3));
        assert_eq!(idiv(&Value::Float(7.5), &Value::Float(2.0)), Value::Int(3));
    }

    #[test]
    fn test_comparisons() {
        // Equality
        assert_eq!(eq(&Value::Int(1), &Value::Int(1)), Value::Bool(true));
        assert_eq!(eq(&Value::Int(1), &Value::Int(2)), Value::Bool(false));

        // CRITICAL: na == na is false
        assert_eq!(eq(&Value::Na, &Value::Na), Value::Bool(false));
        assert_eq!(eq(&Value::Na, &Value::Int(1)), Value::Bool(false));
        assert_eq!(eq(&Value::Int(1), &Value::Na), Value::Bool(false));

        // Inequality
        assert_eq!(ne(&Value::Int(1), &Value::Int(2)), Value::Bool(true));
        assert_eq!(ne(&Value::Na, &Value::Na), Value::Bool(true)); // na != na is true!
        assert_eq!(ne(&Value::Na, &Value::Int(1)), Value::Bool(true));

        // Ordering comparisons with NA return false
        assert_eq!(lt(&Value::Int(1), &Value::Int(2)), Value::Bool(true));
        assert_eq!(lt(&Value::Na, &Value::Int(2)), Value::Bool(false));
        assert_eq!(lt(&Value::Int(1), &Value::Na), Value::Bool(false));
    }

    #[test]
    fn test_logical() {
        // AND
        assert_eq!(
            and(&Value::Bool(true), &Value::Bool(true)),
            Value::Bool(true)
        );
        assert_eq!(
            and(&Value::Bool(true), &Value::Bool(false)),
            Value::Bool(false)
        );
        assert_eq!(
            and(&Value::Bool(false), &Value::Bool(true)),
            Value::Bool(false)
        );

        // false and na = false (short circuit)
        assert_eq!(and(&Value::Bool(false), &Value::Na), Value::Bool(false));
        assert_eq!(and(&Value::Na, &Value::Bool(false)), Value::Bool(false));

        // true and na = na
        assert!(and(&Value::Bool(true), &Value::Na).is_na());

        // OR
        assert_eq!(
            or(&Value::Bool(true), &Value::Bool(false)),
            Value::Bool(true)
        );
        assert_eq!(
            or(&Value::Bool(false), &Value::Bool(false)),
            Value::Bool(false)
        );

        // true or na = true (short circuit)
        assert_eq!(or(&Value::Bool(true), &Value::Na), Value::Bool(true));
        assert_eq!(or(&Value::Na, &Value::Bool(true)), Value::Bool(true));

        // false or na = na
        assert!(or(&Value::Bool(false), &Value::Na).is_na());

        // NOT
        assert_eq!(not(&Value::Bool(true)), Value::Bool(false));
        assert_eq!(not(&Value::Bool(false)), Value::Bool(true));
        assert!(not(&Value::Na).is_na());
    }

    #[test]
    fn test_nz() {
        assert_eq!(nz(&Value::Na, &Value::Int(0)), Value::Int(0));
        assert_eq!(nz(&Value::Int(5), &Value::Int(0)), Value::Int(5));
        assert_eq!(nz(&Value::Float(3.14), &Value::Int(0)), Value::Float(3.14));
    }

    #[test]
    fn test_coalesce() {
        assert_eq!(coalesce(&Value::Na, &Value::Int(1)), Value::Int(1));
        assert_eq!(coalesce(&Value::Int(2), &Value::Int(1)), Value::Int(2));
        assert!(coalesce(&Value::Na, &Value::Na).is_na());
    }

    #[test]
    fn test_if_then_else() {
        assert_eq!(
            if_then_else(&Value::Bool(true), &Value::Int(1), &Value::Int(2)),
            Value::Int(1)
        );
        assert_eq!(
            if_then_else(&Value::Bool(false), &Value::Int(1), &Value::Int(2)),
            Value::Int(2)
        );
        // na condition = na result
        assert!(if_then_else(&Value::Na, &Value::Int(1), &Value::Int(2)).is_na());
    }

    #[test]
    fn test_math() {
        // Power
        assert_eq!(pow(&Value::Int(2), &Value::Int(3)), Value::Float(8.0));
        assert_eq!(
            pow(&Value::Float(4.0), &Value::Float(0.5)),
            Value::Float(2.0)
        );

        // Square root
        assert_eq!(sqrt(&Value::Int(4)), Value::Float(2.0));
        assert_eq!(sqrt(&Value::Int(-1)), Value::Na); // negative sqrt = na

        // Min/Max - preserve input type
        assert_eq!(min(&Value::Int(2), &Value::Int(3)), Value::Int(2));
        assert_eq!(max(&Value::Int(2), &Value::Int(3)), Value::Int(3));
    }

    #[test]
    fn test_bitwise() {
        assert_eq!(
            bit_and(&Value::Int(0b1100), &Value::Int(0b1010)),
            Value::Int(0b1000)
        );
        assert_eq!(
            bit_or(&Value::Int(0b1100), &Value::Int(0b1010)),
            Value::Int(0b1110)
        );
        assert_eq!(
            bit_xor(&Value::Int(0b1100), &Value::Int(0b1010)),
            Value::Int(0b0110)
        );
        assert_eq!(bit_not(&Value::Int(0)), Value::Int(!0i64));
        assert_eq!(shl(&Value::Int(1), &Value::Int(3)), Value::Int(8));
        assert_eq!(shr(&Value::Int(8), &Value::Int(2)), Value::Int(2));
    }
}
