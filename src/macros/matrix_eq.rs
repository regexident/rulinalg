use matrix::BaseMatrix;
use ulp;
use ulp::Ulp;

use libnum::{Num, Float};

use std::fmt;

const MAX_MISMATCH_REPORTS: usize = 12;

#[doc(hidden)]
pub trait ComparisonFailure {
    fn failure_reason(&self) -> Option<String>;
}

#[doc(hidden)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MatrixElementComparisonFailure<T, E> where E: ComparisonFailure {
    pub x: T,
    pub y: T,
    pub error: E,
    pub row: usize,
    pub col: usize
}

impl<T, E> fmt::Display for MatrixElementComparisonFailure<T, E>
    where T: fmt::Display,
          E: ComparisonFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "({i}, {j}): x = {x}, y = {y}.{reason}",
               i = self.row,
               j = self.col,
               x = self.x,
               y = self.y,
               reason = self.error.failure_reason()
                                  // Add a space before the reason
                                  .map(|s| format!(" {}", s))
                                  .unwrap_or(String::new()))
    }
}

#[doc(hidden)]
#[derive(Debug, PartialEq)]
pub enum MatrixComparisonResult<T, C, E>
    where T: Copy,
          C: ElementwiseComparator<T, E>,
          E: ComparisonFailure {
    Match,
    MismatchedDimensions { dim_x: (usize, usize), dim_y: (usize, usize) },
    MismatchedElements { comparator: C, mismatches: Vec<MatrixElementComparisonFailure<T, E>> }
}

/// Trait that describes elementwise comparators for [assert_matrix_eq!](../macro.assert_matrix_eq!.html).
///
/// Usually you should not need to interface with this trait directly. It is a part of the documentation
/// only so that the trait bounds for the comparators are made public.
pub trait ElementwiseComparator<T, E> where T: Copy, E: ComparisonFailure {
    /// Compares two elements.
    ///
    /// Returns the error associated with the comparison if it failed.
    fn compare(&self, x: T, y: T) -> Result<(), E>;

    /// A description of the comparator.
    fn description(&self) -> String;
}

impl<T, C, E> MatrixComparisonResult<T, C, E>
    where T: Copy + fmt::Display,
          C: ElementwiseComparator<T, E>,
          E: ComparisonFailure {
    pub fn panic_message(&self) -> Option<String> {

        match self {
            &MatrixComparisonResult::MismatchedElements { ref comparator, ref mismatches } => {
                // TODO: Aligned output
                let mut formatted_mismatches = String::new();

                let mismatches_overflow = mismatches.len() > MAX_MISMATCH_REPORTS;
                let overflow_msg = if mismatches_overflow {
                    let num_hidden_entries = mismatches.len() - MAX_MISMATCH_REPORTS;
                    format!(" ... ({} mismatching elements not shown)\n", num_hidden_entries)
                } else {
                    String::new()
                };

                for mismatch in mismatches.iter().take(MAX_MISMATCH_REPORTS) {
                    formatted_mismatches.push_str(" ");
                    formatted_mismatches.push_str(&mismatch.to_string());
                    formatted_mismatches.push_str("\n");
                }

                // Strip off the last newline from the above
                formatted_mismatches = formatted_mismatches.trim_right().to_string();

                Some(format!("\n
Matrices X and Y have {num} mismatched element pairs.
The mismatched elements are listed below, in the format
(row, col): x = X[[row, col]], y = Y[[row, col]].

{mismatches}
{overflow_msg}
Comparison criterion: {description}
\n",
                    num = mismatches.len(),
                    description = comparator.description(),
                    mismatches = formatted_mismatches,
                    overflow_msg = overflow_msg))
            },
            &MatrixComparisonResult::MismatchedDimensions { dim_x, dim_y } => {
                Some(format!("\n
Dimensions of matrices X and Y do not match.
 dim(X) = {x_rows} x {x_cols}
 dim(Y) = {y_rows} x {y_cols}
\n",
                    x_rows = dim_x.0, x_cols = dim_x.1,
                    y_rows = dim_y.0, y_cols = dim_y.1))
            },
            _ => None
        }
    }
}

#[doc(hidden)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VectorElementComparisonFailure<T, E> where E: ComparisonFailure {
    pub x: T,
    pub y: T,
    pub error: E,
    pub index: usize
}

impl<T, E> fmt::Display for VectorElementComparisonFailure<T, E>
    where T: fmt::Display, E: ComparisonFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "#{index}: x = {x}, y = {y}.{reason}",
               index = self.index,
               x = self.x,
               y = self.y,
               reason = self.error.failure_reason()
                                  // Add a space before the reason
                                  .map(|s| format!(" {}", s))
                                  .unwrap_or(String::new()))
    }
}

#[doc(hidden)]
#[derive(Debug, PartialEq)]
pub enum VectorComparisonResult<T, C, E>
    where T: Copy,
          C: ElementwiseComparator<T, E>,
          E: ComparisonFailure {
    Match,
    MismatchedDimensions {
        dim_x: usize,
        dim_y: usize
    },
    MismatchedElements {
        comparator: C,
        mismatches: Vec<VectorElementComparisonFailure<T, E>>
    }
}

impl <T, C, E> VectorComparisonResult<T, C, E>
    where T: Copy + fmt::Display, C: ElementwiseComparator<T, E>, E: ComparisonFailure {
    pub fn panic_message(&self) -> Option<String> {
        match self {
            &VectorComparisonResult::MismatchedElements { ref comparator, ref mismatches } => {
                let mut formatted_mismatches = String::new();

                let mismatches_overflow = mismatches.len() > MAX_MISMATCH_REPORTS;
                let overflow_msg = if mismatches_overflow {
                    let num_hidden_entries = mismatches.len() - MAX_MISMATCH_REPORTS;
                    format!(" ... ({} mismatching elements not shown)\n", num_hidden_entries)
                } else {
                    String::new()
                };

                for mismatch in mismatches.iter().take(MAX_MISMATCH_REPORTS) {
                    formatted_mismatches.push_str(" ");
                    formatted_mismatches.push_str(&mismatch.to_string());
                    formatted_mismatches.push_str("\n");
                }

                // Strip off the last newline from the above
                formatted_mismatches = formatted_mismatches.trim_right().to_string();

                Some(format!("\n
Vectors X and Y have {num} mismatched element pairs.
The mismatched elements are listed below, in the format
#index: x = X[index], y = Y[index].

{mismatches}
{overflow_msg}
Comparison criterion: {description}
\n",
                    num = mismatches.len(),
                    description = comparator.description(),
                    mismatches = formatted_mismatches,
                    overflow_msg = overflow_msg))
            },
            &VectorComparisonResult::MismatchedDimensions { dim_x, dim_y } => {
                Some(format!("\n
Dimensions of vectors X and Y do not match.
 dim(X) = {dim_x}
 dim(Y) = {dim_y}
\n",
                    dim_x = dim_x,
                    dim_y = dim_y))
            },
            _ => None
        }
    }
}

#[doc(hidden)]
pub fn elementwise_matrix_comparison<T, M, C, E>(x: &M, y: &M, comparator: C)
    -> MatrixComparisonResult<T, C, E>
    where M: BaseMatrix<T>, T: Copy, C: ElementwiseComparator<T, E>, E: ComparisonFailure {
    if x.rows() == y.rows() && x.cols() == y.cols() {
        let mismatches = {
            let mut mismatches = Vec::new();
            let x = x.as_slice();
            let y = y.as_slice();
            for i in 0 .. x.rows() {
                for j in 0 .. x.cols() {
                    let a = x[[i, j]].to_owned();
                    let b = y[[i, j]].to_owned();
                    if let Err(error) = comparator.compare(a, b) {
                        mismatches.push(MatrixElementComparisonFailure {
                            x: a,
                            y: b,
                            error: error,
                            row: i,
                            col: j
                        });
                    }
                }
            }
            mismatches
        };

        if mismatches.is_empty() {
            MatrixComparisonResult::Match
        } else {
            MatrixComparisonResult::MismatchedElements {
                comparator: comparator,
                mismatches: mismatches
            }
        }
    } else {
        MatrixComparisonResult::MismatchedDimensions {
            dim_x: (x.rows(), x.cols()),
            dim_y: (y.rows(), y.cols())
        }
    }
}

#[doc(hidden)]
pub fn elementwise_vector_comparison<T, C, E>(x: &[T], y: &[T], comparator: C)
    -> VectorComparisonResult<T, C, E>
    where T: Copy,
          C: ElementwiseComparator<T, E>,
          E: ComparisonFailure {
    // The reason this function takes a slice and not a Vector ref,
    // is that we the assert_vector_eq! macro to work with both
    // references and owned values
    if x.len() == y.len() {
        let n = x.len();
        let mismatches = {
            let mut mismatches = Vec::new();
            for i in 0 .. n {
                let a = x[i].to_owned();
                let b = y[i].to_owned();
                if let Err(error) = comparator.compare(a, b) {
                    mismatches.push(VectorElementComparisonFailure {
                        x: a,
                        y: b,
                        error: error,
                        index: i
                    });
                }
            }
            mismatches
        };

        if mismatches.is_empty() {
            VectorComparisonResult::Match
        } else {
            VectorComparisonResult::MismatchedElements {
                comparator: comparator,
                mismatches: mismatches
            }
        }
    } else {
        VectorComparisonResult::MismatchedDimensions { dim_x: x.len(), dim_y: y.len() }
    }
}

#[doc(hidden)]
#[derive(Copy, Clone, Debug, PartialEq)]
struct AbsoluteError<T>(pub T);

/// The `abs` comparator used with [assert_matrix_eq!](../macro.assert_matrix_eq!.html).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AbsoluteElementwiseComparator<T> {
    /// The maximum absolute difference tolerated (inclusive).
    pub tol: T
}

impl<T> ComparisonFailure for AbsoluteError<T> where T: fmt::Display {
    fn failure_reason(&self) -> Option<String> {
        Some(
            format!("Absolute error: {error}.", error = self.0)
        )
    }
}

impl<T> ElementwiseComparator<T, AbsoluteError<T>> for AbsoluteElementwiseComparator<T>
    where T: Copy + fmt::Display + Num + PartialOrd<T> {

    fn compare(&self, a: T, b: T) -> Result<(), AbsoluteError<T>> {
        assert!(self.tol >= T::zero());

        // Note: Cannot use num::abs because we do not want to restrict
        // ourselves to Signed types (i.e. we still want to be able to
        // handle unsigned types).

        if a == b {
            Ok(())
        } else {
            let distance = if a > b { a - b } else { b - a };
            if distance <= self.tol {
                Ok(())
            } else {
                Err(AbsoluteError(distance))
            }
        }
    }

    fn description(&self) -> String {
        format!("absolute difference, |x - y| <= {tol}.", tol = self.tol)
    }
}

/// The `exact` comparator used with [assert_matrix_eq!](../macro.assert_matrix_eq!.html).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ExactElementwiseComparator;

#[doc(hidden)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ExactError;

impl ComparisonFailure for ExactError {
    fn failure_reason(&self) -> Option<String> { None }
}

impl<T> ElementwiseComparator<T, ExactError> for ExactElementwiseComparator
    where T: Copy + fmt::Display + PartialEq<T> {

    fn compare(&self, a: T, b: T) -> Result<(), ExactError> {
        if a == b {
            Ok(())
        } else {
            Err(ExactError)
        }
    }

    fn description(&self) -> String {
        format!("exact equality x == y.")
    }
}

/// The `ulp` comparator used with [assert_matrix_eq!](../macro.assert_matrix_eq!.html).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UlpElementwiseComparator {
    /// The maximum difference in ULP units tolerated (inclusive).
    pub tol: u64
}

#[doc(hidden)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UlpError(pub ulp::UlpComparisonResult);

impl ComparisonFailure for UlpError {
    fn failure_reason(&self) -> Option<String> {
        use ulp::UlpComparisonResult;
        match self.0 {
            UlpComparisonResult::Difference(diff) =>
                Some(format!("Difference: {diff} ULP.", diff=diff)),
            UlpComparisonResult::IncompatibleSigns =>
                Some(format!("Numbers have incompatible signs.")),
            _ => None
        }
    }
}

impl<T> ElementwiseComparator<T, UlpError> for UlpElementwiseComparator
    where T: Copy + Ulp {

    fn compare(&self, a: T, b: T) -> Result<(), UlpError> {
        let diff = Ulp::ulp_diff(&a, &b);
        match diff {
            ulp::UlpComparisonResult::ExactMatch => Ok(()),
            ulp::UlpComparisonResult::Difference(diff) if diff <= self.tol => Ok(()),
            _ => Err(UlpError(diff))
        }
    }

    fn description(&self) -> String {
        format!("ULP difference less than or equal to {tol}. See documentation for details.",
                tol = self.tol)
    }
}

/// The `float` comparator used with [assert_matrix_eq!](../macro.assert_matrix_eq!.html).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FloatElementwiseComparator<T> {
    abs: AbsoluteElementwiseComparator<T>,
    ulp: UlpElementwiseComparator
}

#[doc(hidden)]
#[allow(dead_code)]
impl<T> FloatElementwiseComparator<T> where T: Float + Ulp {
    pub fn default() -> Self {
        FloatElementwiseComparator {
            abs: AbsoluteElementwiseComparator { tol: T::epsilon() },
            ulp: UlpElementwiseComparator { tol: 4 }
        }
    }

    pub fn eps(self, eps: T) -> Self {
        FloatElementwiseComparator {
            abs: AbsoluteElementwiseComparator { tol: eps },
            ulp: self.ulp
        }
    }

    pub fn ulp(self, max_ulp: u64) -> Self {
        FloatElementwiseComparator {
            abs: self.abs,
            ulp: UlpElementwiseComparator { tol: max_ulp }
        }
    }
}

impl<T> ElementwiseComparator<T, UlpError> for FloatElementwiseComparator<T>
    where T: Copy + Ulp + Float + fmt::Display {
    fn compare(&self, a: T, b: T) -> Result<(), UlpError> {
        // First perform an absolute comparison with a presumably very small epsilon tolerance
        if let Err(_) = self.abs.compare(a, b) {
            // Then fall back to an ULP-based comparison
            self.ulp.compare(a, b)
        } else {
            // If the epsilon comparison succeeds, we have a match
             Ok(())
        }
    }

    fn description(&self) -> String {
        format!("
Epsilon-sized absolute comparison, followed by an ULP-based comparison.
Please see the documentation for details.
Epsilon:       {eps}
ULP tolerance: {ulp}",
            eps = self.abs.tol,
            ulp = self.ulp.tol)
    }
}


/// Compare matrices for exact or approximate equality.
///
/// The `assert_matrix_eq!` simplifies the comparison of two matrices by
/// providing the following features:
///
/// - Verifies that the dimensions of the matrices match.
/// - Offers both exact and approximate comparison of individual elements.
/// - Multiple types of comparators available, depending on the needs of the user.
/// - Built-in error reporting makes it easy to determine which elements of the two matrices
///   that do not compare equal.
///
/// # Usage
/// Given two matrices `x` and `y`, the default invocation performs an exact elementwise
/// comparison of the two matrices.
///
/// ```
/// # #[macro_use] extern crate rulinalg; fn main() { let x = matrix![1.0f64]; let y = matrix![1.0f64];
/// // Performs elementwise exact comparison
/// assert_matrix_eq!(x, y);
/// # }
/// ```
///
/// An exact comparison is often not desirable. In particular, with floating point types,
/// rounding errors or other sources of inaccuracies tend to complicate the matter.
/// For this purpose, `assert_matrix_eq!` provides several comparators.
///
/// ```
/// # #[macro_use] extern crate rulinalg; fn main() {
/// # let x = matrix![1.0f64]; let y = matrix![1.0f64];
/// // Available comparators:
/// assert_matrix_eq!(x, y, comp = exact);
/// assert_matrix_eq!(x, y, comp = float);
/// assert_matrix_eq!(x, y, comp = abs, tol = 1e-12);
/// assert_matrix_eq!(x, y, comp = ulp, tol = 8);
/// # }
/// ```
/// **Note**: The `comp` argument *must* be specified after `x` and `y`, and cannot come
/// after comparator-specific options. This is a deliberate design decision,
/// with the rationale that assertions should look as uniform as possible for
/// the sake of readability.
///
///
/// ### The `exact` comparator
/// This comparator simply uses the default `==` operator to compare each pair of elements.
/// The default comparator delegates the comparison to the `exact` comparator.
///
/// ### The `float` comparator
/// The `float` comparator is designed to be a conservative default for comparing floating-point numbers.
/// It is inspired by the `AlmostEqualUlpsAndAbs` comparison function proposed in the excellent blog post
/// [Comparing Floating Point Numbers, 2012 Edition]
/// (https://randomascii.wordpress.com/2012/02/25/comparing-floating-point-numbers-2012-edition/)
/// by Bruce Dawson.
///
/// If you expect the two matrices to be almost exactly the same, but you want to leave some
/// room for (very small) rounding errors, then this comparator should be your default choice.
///
/// The comparison criterion can be summarized as follows:
///
/// 1. If `assert_matrix_eq!(x, y, comp = abs, tol = max_eps)` holds for `max_eps` close to the
///    machine epsilon for the floating point type,
///    then the comparison is successful.
/// 2. Otherwise, returns the result of `assert_matrix_eq!(x, y, comp = ulp, tol = max_ulp)`,
///    where `max_ulp` is a small positive integer constant.
///
/// The `max_eps` and `max_ulp` parameters can be tweaked to your preference with the syntax:
///
/// ```
/// # #[macro_use] extern crate rulinalg; fn main() {
/// # let x = matrix![1.0f64]; let y = matrix![1.0f64];
/// # let max_eps = 1.0; let max_ulp = 0;
/// assert_matrix_eq!(x, y, comp = float, eps = max_eps, ulp = max_ulp);
/// # }
/// ```
///
/// These additional parameters can be specified in any order after the choice of comparator,
/// and do not both need to be present.
///
/// ### The `abs` comparator
/// Compares the absolute difference between individual elements against the specified tolerance.
/// Specifically, for every pair of elements x and y picked from the same row and column in X and Y
/// respectively, the criterion is defined by
///
/// ```text
///     | x - y | <= tol.
/// ```
///
/// In addition to floating point numbers, the comparator can also be used for integral numbers,
/// both signed and unsigned. In order to avoid unsigned underflow, the difference is always
/// computed by subtracting the smaller number from the larger number.
/// Note that the type of `tol` is required to be the same as that of the scalar field.
///
///
/// ### The `ulp` comparator
/// Elementwise comparison of floating point numbers based on their
/// [ULP](https://en.wikipedia.org/wiki/Unit_in_the_last_place) difference.
/// Once again, this is inspired by the proposals
/// [in the aforementioned blog post by Bruce Dawon]
/// (https://randomascii.wordpress.com/2012/02/25/comparing-floating-point-numbers-2012-edition/),
/// but it handles some cases explicitly as to provide better error reporting.
///
/// Note that the ULP difference of two floating point numbers is not defined in the following cases:
///
/// - The two numbers have different signs. The only exception here is +0 and -0,
///   which are considered an exact match.
/// - One of the numbers is NaN.
///
/// ULP-based comparison is typically used when two numbers are expected to be very,
/// very close to each other. However, it is typically not very useful very close to zero,
/// which is discussed in the linked blog post above.
/// The error in many mathematical functions can often be bounded by a certain number of ULP, and so
/// this comparator is particularly useful if this number is known.
///
/// Note that the scalar type of the matrix must implement the [Ulp trait](ulp/trait.Ulp.html) in order
/// to be used with this comparator. By default, `f32` and `f64` implementations are provided.
///
/// # Error reporting
///
/// One of the main motivations for the `assert_matrix_eq!` macro is the ability to give
/// useful error messages which help pinpoint the problems. For example, consider the example
///
/// ```rust,should_panic
/// #[macro_use]
/// extern crate rulinalg;
///
/// fn main() {
///     let a = matrix![1.00, 2.00;
///                     3.00, 4.00];
///     let b = matrix![1.01, 2.00;
///                     3.40, 4.00];
///     assert_matrix_eq!(a, b, comp = abs, tol = 1e-8);
/// }
/// ```
///
/// which yields the output
///
/// ```text
/// Matrices X and Y have 2 mismatched element pairs.
/// The mismatched elements are listed below, in the format
/// (row, col): x = X[[row, col]], y = Y[[row, col]].
///
/// (0, 0): x = 1, y = 1.01. Absolute error: 0.010000000000000009.
/// (1, 0): x = 3, y = 3.4. Absolute error: 0.3999999999999999.
///
/// Comparison criterion: absolute difference, |x - y| <= 0.00000001.
/// ```
///
/// # Trait bounds on elements
/// Each comparator has specific requirements on which traits the elements
/// need to implement. To discover which traits are required for each comparator,
/// we refer the reader to implementors of
/// [ElementwiseComparator](macros/trait.ElementwiseComparator.html),
/// which provides the underlying comparison for the various macro invocations.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate rulinalg;
/// use rulinalg::matrix::Matrix;
///
/// # fn main() {
/// let ref a = matrix![1, 2;
///                 3, 4i64];
/// let ref b = matrix![1, 3;
///                 3, 4i64];
///
/// let ref x = matrix![1.000, 2.000,
///                 3.000, 4.000f64];
/// let ref y = matrix![0.999, 2.001,
///                 2.998, 4.000f64];
///
///
/// // comp = abs is also applicable to integers
/// assert_matrix_eq!(a, b, comp = abs, tol = 1);
/// assert_matrix_eq!(x, y, comp = abs, tol = 0.01);
///
/// assert_matrix_eq!(a * 2, a + a);
/// assert_matrix_eq!(x * 2.0, x + x, comp = float);
/// # }
/// ```
#[macro_export]
macro_rules! assert_matrix_eq {
    ($x:expr, $y:expr) => {
        {
            // Note: The reason we take slices of both x and y is that if x or y are passed as references,
            // we don't attempt to call elementwise_matrix_comparison with a &&BaseMatrix type (double reference),
            // which does not work due to generics.
            use $crate::macros::{elementwise_matrix_comparison, ExactElementwiseComparator};
            use $crate::matrix::BaseMatrix;
            let comp = ExactElementwiseComparator;
            let msg = elementwise_matrix_comparison(&$x.as_slice(), &$y.as_slice(), comp).panic_message();
            if let Some(msg) = msg {
                // Note: We need the panic to incur here inside of the macro in order
                // for the line number to be correct when using it for tests,
                // hence we build the panic message in code, but panic here.
                panic!("{msg}
Please see the documentation for ways to compare matrices approximately.\n\n",
                    msg = msg.trim_right());
            }
        }
    };
    ($x:expr, $y:expr, comp = exact) => {
        {
            use $crate::macros::{elementwise_matrix_comparison, ExactElementwiseComparator};
            use $crate::matrix::BaseMatrix;
            let comp = ExactElementwiseComparator;
            let msg = elementwise_matrix_comparison(&$x.as_slice(), &$y.as_slice(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    ($x:expr, $y:expr, comp = abs, tol = $tol:expr) => {
        {
            use $crate::macros::{elementwise_matrix_comparison, AbsoluteElementwiseComparator};
            use $crate::matrix::BaseMatrix;
            let comp = AbsoluteElementwiseComparator { tol: $tol };
            let msg = elementwise_matrix_comparison(&$x.as_slice(), &$y.as_slice(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    ($x:expr, $y:expr, comp = ulp, tol = $tol:expr) => {
        {
            use $crate::macros::{elementwise_matrix_comparison, UlpElementwiseComparator};
            use $crate::matrix::BaseMatrix;
            let comp = UlpElementwiseComparator { tol: $tol };
            let msg = elementwise_matrix_comparison(&$x.as_slice(), &$y.as_slice(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    ($x:expr, $y:expr, comp = float) => {
        {
            use $crate::macros::{elementwise_matrix_comparison, FloatElementwiseComparator};
            use $crate::matrix::BaseMatrix;
            let comp = FloatElementwiseComparator::default();
            let msg = elementwise_matrix_comparison(&$x.as_slice(), &$y.as_slice(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    // This following allows us to optionally tweak the epsilon and ulp tolerances
    // used in the default float comparator.
    ($x:expr, $y:expr, comp = float, $($key:ident = $val:expr),+) => {
        {
            use $crate::macros::{elementwise_matrix_comparison, FloatElementwiseComparator};
            use $crate::matrix::BaseMatrix;
            let comp = FloatElementwiseComparator::default()$(.$key($val))+;
            let msg = elementwise_matrix_comparison(&$x.as_slice(), &$y.as_slice(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
}

/// Compare vectors for exact or approximate equality.
///
/// This macro works analogously to [assert_matrix_eq!](macro.assert_matrix_eq.html),
/// but is used for comparing instances of [Vector](vector/struct.Vector.html) rather than
/// matrices.
#[macro_export]
macro_rules! assert_vector_eq {
    ($x:expr, $y:expr) => {
        {
            // Note: The reason we take slices of both x and y is that if x or y are passed as references,
            // we don't attempt to call elementwise_matrix_comparison with a &&BaseMatrix type (double reference),
            // which does not work due to generics.
            use $crate::macros::{elementwise_vector_comparison, ExactElementwiseComparator};
            let comp = ExactElementwiseComparator;
            let msg = elementwise_vector_comparison($x.data(), $y.data(), comp).panic_message();
            if let Some(msg) = msg {
                // Note: We need the panic to incur here inside of the macro in order
                // for the line number to be correct when using it for tests,
                // hence we build the panic message in code, but panic here.
                panic!("{msg}
Please see the documentation for ways to compare vectors approximately.\n\n",
                    msg = msg.trim_right());
            }
        }
    };
    ($x:expr, $y:expr, comp = exact) => {
        {
            use $crate::macros::{elementwise_vector_comparison, ExactElementwiseComparator};
            let comp = ExactElementwiseComparator;
            let msg = elementwise_vector_comparison($x.data(), $y.data(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    ($x:expr, $y:expr, comp = abs, tol = $tol:expr) => {
        {
            use $crate::macros::{elementwise_vector_comparison, AbsoluteElementwiseComparator};
            let comp = AbsoluteElementwiseComparator { tol: $tol };
            let msg = elementwise_vector_comparison($x.data(), $y.data(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    ($x:expr, $y:expr, comp = ulp, tol = $tol:expr) => {
        {
            use $crate::macros::{elementwise_vector_comparison, UlpElementwiseComparator};
            let comp = UlpElementwiseComparator { tol: $tol };
            let msg = elementwise_vector_comparison($x.data(), $y.data(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    ($x:expr, $y:expr, comp = float) => {
        {
            use $crate::macros::{elementwise_vector_comparison, FloatElementwiseComparator};
            let comp = FloatElementwiseComparator::default();
            let msg = elementwise_vector_comparison($x.data(), $y.data(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
    // This following allows us to optionally tweak the epsilon and ulp tolerances
    // used in the default float comparator.
    ($x:expr, $y:expr, comp = float, $($key:ident = $val:expr),+) => {
        {
            use $crate::macros::{elementwise_vector_comparison, FloatElementwiseComparator};
            let comp = FloatElementwiseComparator::default()$(.$key($val))+;
            let msg = elementwise_vector_comparison($x.data(), $y.data(), comp).panic_message();
            if let Some(msg) = msg {
                panic!(msg);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::{AbsoluteElementwiseComparator, AbsoluteError, ElementwiseComparator,
        ExactElementwiseComparator, ExactError,
        UlpElementwiseComparator, UlpError,
        FloatElementwiseComparator,
        elementwise_matrix_comparison,
        elementwise_vector_comparison,
        MatrixComparisonResult,
        VectorComparisonResult};
    use matrix::Matrix;
    use vector::Vector;
    use ulp::{Ulp, UlpComparisonResult};
    use quickcheck::TestResult;
    use std::f64;

    /// Returns the next adjacent floating point number (in the direction of positive infinity)
    fn next_f64(x: f64) -> f64 {
        use std::mem;
        let as_int = unsafe { mem::transmute::<f64, i64>(x) };
        unsafe { mem::transmute::<i64, f64>(as_int + 1) }
    }

    #[test]
    pub fn absolute_comparator_integer() {
        let comp = AbsoluteElementwiseComparator { tol: 1 };

        assert_eq!(comp.compare(0, 0), Ok(()));
        assert_eq!(comp.compare(1, 0), Ok(()));
        assert_eq!(comp.compare(-1, 0), Ok(()));
        assert_eq!(comp.compare(2, 0), Err(AbsoluteError(2)));
        assert_eq!(comp.compare(-2, 0), Err(AbsoluteError(2)));
    }

    #[test]
    pub fn absolute_comparator_floating_point() {
        let comp = AbsoluteElementwiseComparator { tol: 1.0 };

        // Note: floating point math is not generally exact, but
        // here we only compare with 0.0, so we can expect exact results.
        assert_eq!(comp.compare(0.0, 0.0), Ok(()));
        assert_eq!(comp.compare(1.0, 0.0), Ok(()));
        assert_eq!(comp.compare(-1.0, 0.0), Ok(()));
        assert_eq!(comp.compare(2.0, 0.0), Err(AbsoluteError(2.0)));
        assert_eq!(comp.compare(-2.0, 0.0), Err(AbsoluteError(2.0)));
    }

    quickcheck! {
        fn property_absolute_comparator_is_symmetric_i64(a: i64, b: i64, tol: i64) -> TestResult {
            if tol <= 0 {
                return TestResult::discard()
            }

            let comp = AbsoluteElementwiseComparator { tol: tol };
            TestResult::from_bool(comp.compare(a, b) == comp.compare(b, a))
        }
    }

    quickcheck! {
        fn property_absolute_comparator_is_symmetric_f64(a: f64, b: f64, tol: f64) -> TestResult {
            if tol <= 0.0 {
                return TestResult::discard()
            }

            // Floating point math is not exact, but the AbsoluteElementwiseComparator is designed
            // so that it gives exactly the same result when the argument positions are reversed
            let comp = AbsoluteElementwiseComparator { tol: tol };
            TestResult::from_bool(comp.compare(a, b) == comp.compare(b, a))
        }
    }

    quickcheck! {
        fn property_absolute_comparator_tolerance_is_not_strict_f64(tol: f64) -> TestResult {
            if tol <= 0.0 || !tol.is_finite() {
                return TestResult::discard()
            }

            // The comparator is defined by <=, not <
            let comp = AbsoluteElementwiseComparator { tol: tol };
            let includes_tol = comp.compare(tol, 0.0).is_ok();
            let excludes_next_after_tol = comp.compare(next_f64(tol), 0.0).is_err();
            TestResult::from_bool(includes_tol && excludes_next_after_tol)
        }
    }

    #[test]
    pub fn exact_comparator_integer() {
        let comp = ExactElementwiseComparator;

        assert_eq!(comp.compare(0, 0), Ok(()));
        assert_eq!(comp.compare(1, 0), Err(ExactError));
        assert_eq!(comp.compare(-1, 0), Err(ExactError));
        assert_eq!(comp.compare(1, -1), Err(ExactError));
    }

    #[test]
    pub fn exact_comparator_floating_point() {
        let comp = ExactElementwiseComparator;

        assert_eq!(comp.compare(0.0, 0.0), Ok(()));
        assert_eq!(comp.compare(-0.0, -0.0), Ok(()));
        assert_eq!(comp.compare(-0.0, 0.0), Ok(()));
        assert_eq!(comp.compare(1.0, 0.0), Err(ExactError));
        assert_eq!(comp.compare(-1.0, 0.0), Err(ExactError));
        assert_eq!(comp.compare(f64::NAN, 5.0), Err(ExactError));
    }

    quickcheck! {
        fn property_exact_comparator_is_symmetric_i64(a: i64, b: i64) -> bool {
            let comp = ExactElementwiseComparator;
            comp.compare(a, b) == comp.compare(b, a)
        }
    }

    quickcheck! {
        fn property_exact_comparator_is_symmetric_f64(a: f64, b: f64) -> bool {
            let comp = ExactElementwiseComparator;
            comp.compare(a, b) == comp.compare(b, a)
        }
    }

    quickcheck! {
        fn property_exact_comparator_matches_equality_operator_i64(a: i64, b: i64) -> bool {
            let comp = ExactElementwiseComparator;
            let result = comp.compare(a, b);

            match a == b {
                true =>  result == Ok(()),
                false => result == Err(ExactError)
            }
        }
    }

    quickcheck! {
        fn property_exact_comparator_matches_equality_operator_f64(a: f64, b: f64) -> bool {
            let comp = ExactElementwiseComparator;
            let result = comp.compare(a, b);

            match a == b {
                true =>  result == Ok(()),
                false => result == Err(ExactError)
            }
        }
    }

    #[test]
    pub fn ulp_comparator_f64() {
        // The Ulp implementation has its own set of tests, so we just want
        // to make a sample here
        let comp = UlpElementwiseComparator { tol: 1 };

        assert_eq!(comp.compare(0.0, 0.0), Ok(()));
        assert_eq!(comp.compare(0.0, -0.0), Ok(()));
        assert_eq!(comp.compare(-1.0, 1.0), Err(UlpError(UlpComparisonResult::IncompatibleSigns)));
        assert_eq!(comp.compare(1.0, 0.0), Err(UlpError(f64::ulp_diff(&1.0, &0.0))));
        assert_eq!(comp.compare(f64::NAN, 0.0), Err(UlpError(UlpComparisonResult::Nan)));;
    }

    quickcheck! {
        fn property_ulp_comparator_is_symmetric(a: f64, b: f64, tol: u64) -> TestResult {
            if tol == 0 {
                return TestResult::discard()
            }

            let comp = UlpElementwiseComparator { tol: tol };
            TestResult::from_bool(comp.compare(a, b) == comp.compare(b, a))
        }
    }

    quickcheck! {
        fn property_ulp_comparator_matches_ulp_trait(a: f64, b: f64, tol: u64) -> bool {
            let comp = UlpElementwiseComparator { tol: tol };
            let result = comp.compare(a, b);

            use ulp::UlpComparisonResult::{ExactMatch, Difference};

            match f64::ulp_diff(&a, &b) {
                ExactMatch =>                      result.is_ok(),
                Difference(diff) if diff <= tol => result.is_ok(),
                otherwise =>                       result == Err(UlpError(otherwise))
            }
        }
    }

    quickcheck! {
        fn property_ulp_comparator_next_f64_is_ok_when_inside_tolerance(x: f64) -> TestResult {
            let y = next_f64(x);

            if !(x.is_finite() && y.is_finite() && x.signum() == y.signum()) {
                return TestResult::discard()
            }

            let comp0 = UlpElementwiseComparator { tol: 0 };
            let comp1 = UlpElementwiseComparator { tol: 1 };

            let tol_0_fails = comp0.compare(x, y) == Err(UlpError(UlpComparisonResult::Difference(1)));
            let tol_1_succeeds = comp1.compare(x, y) == Ok(());

            TestResult::from_bool(tol_0_fails && tol_1_succeeds)
        }
    }

    quickcheck! {
        fn property_float_comparator_matches_abs_with_zero_ulp_tol(a: f64, b: f64, abstol: f64) -> TestResult {
            if abstol <= 0.0 {
                return TestResult::discard()
            }

            let abstol = abstol.abs();
            let comp = FloatElementwiseComparator::default().eps(abstol).ulp(0);
            let abscomp = AbsoluteElementwiseComparator { tol: abstol };
            let result = comp.compare(a, b);

            // Recall that the float comparator returns UlpError, so we cannot compare the results
            // of abscomp directly
            TestResult::from_bool(match abscomp.compare(a, b) {
                Err(AbsoluteError(_)) =>   result.is_err(),
                Ok(_) =>                   result.is_ok()
            })
        }
    }

    quickcheck! {
        fn property_float_comparator_matches_ulp_with_zero_eps_tol(a: f64, b: f64, max_ulp: u64) -> bool {
            let comp = FloatElementwiseComparator::default().eps(0.0).ulp(max_ulp);
            let ulpcomp = UlpElementwiseComparator { tol: max_ulp };

            comp.compare(a, b) == ulpcomp.compare(a, b)
        }
    }

    quickcheck! {
        fn property_elementwise_comparison_incompatible_matrices_yield_dimension_mismatch(
            m: usize,
            n: usize,
            p: usize,
            q: usize) -> TestResult {
            if m == p && n == q {
                return TestResult::discard()
            }

            // It does not actually matter which comparator we use here, but we need to pick one
            let comp = ExactElementwiseComparator;
            let ref x = Matrix::new(m, n, vec![0; m * n]);
            let ref y = Matrix::new(p, q, vec![0; p * q]);

            let expected = MatrixComparisonResult::MismatchedDimensions { dim_x: (m, n), dim_y: (p, q) };

            TestResult::from_bool(elementwise_matrix_comparison(x, y, comp) == expected)
        }
    }

    quickcheck! {
        fn property_elementwise_comparison_matrix_matches_self(m: usize, n: usize) -> bool {
            let comp = ExactElementwiseComparator;
            let ref x = Matrix::new(m, n, vec![0; m * n]);

            elementwise_matrix_comparison(x, x, comp) == MatrixComparisonResult::Match
        }
    }

    #[test]
    fn elementwise_matrix_comparison_reports_correct_mismatches() {
        use super::MatrixComparisonResult::MismatchedElements;
        use super::MatrixElementComparisonFailure;

        let comp = ExactElementwiseComparator;

        {
            // Single element matrices
            let ref x = matrix![1];
            let ref y = matrix![2];

            let expected = MismatchedElements {
                comparator: comp,
                mismatches: vec![MatrixElementComparisonFailure {
                    x: 1, y: 2,
                    error: ExactError,
                    row: 0, col: 0
                }]
            };

            assert_eq!(elementwise_matrix_comparison(x, y, comp), expected);
        }

        {
            // Mismatch in top-left and bottom-corner elements for a short matrix
            let ref x = matrix![0, 1, 2;
                                3, 4, 5];
            let ref y = matrix![1, 1, 2;
                                3, 4, 6];
            let mismatches = vec![
                MatrixElementComparisonFailure {
                    x: 0, y: 1,
                    error: ExactError,
                    row: 0, col: 0
                },
                MatrixElementComparisonFailure {
                    x: 5, y: 6,
                    error: ExactError,
                    row: 1, col: 2
                }
            ];

            let expected = MismatchedElements {
                comparator: comp,
                mismatches: mismatches
            };

            assert_eq!(elementwise_matrix_comparison(x, y, comp), expected);
        }

        {
            // Mismatch in top-left and bottom-corner elements for a tall matrix
            let ref x = matrix![0, 1;
                                2, 3;
                                4, 5];
            let ref y = matrix![1, 1;
                                2, 3;
                                4, 6];
            let mismatches = vec![
                MatrixElementComparisonFailure {
                    x: 0, y: 1,
                    error: ExactError,
                    row: 0, col: 0
                },
                MatrixElementComparisonFailure {
                    x: 5, y: 6,
                    error: ExactError,
                    row: 2, col: 1
                }
            ];

            let expected = MismatchedElements {
                comparator: comp,
                mismatches: mismatches
            };

            assert_eq!(elementwise_matrix_comparison(x, y, comp), expected);
        }

        {
            // Check some arbitrary elements
            let ref x = matrix![0, 1, 2, 3;
                                4, 5, 6, 7];
            let ref y = matrix![0, 1, 3, 3;
                                4, 6, 6, 7];

            let mismatches = vec![
                MatrixElementComparisonFailure {
                    x: 2, y: 3,
                    error: ExactError,
                    row: 0, col: 2
                },
                MatrixElementComparisonFailure {
                    x: 5, y: 6,
                    error: ExactError,
                    row: 1, col: 1
                }
            ];

            let expected = MismatchedElements {
                comparator: comp,
                mismatches: mismatches
            };

            assert_eq!(elementwise_matrix_comparison(x, y, comp), expected);
        }
    }

    #[test]
    pub fn matrix_eq_absolute_compare_self_for_integer() {
        let x = matrix![1, 2, 3;
                        4, 5, 6];
        assert_matrix_eq!(x, x, comp = abs, tol = 0);
    }

    #[test]
    pub fn matrix_eq_absolute_compare_self_for_floating_point() {
        let x = matrix![1.0, 2.0, 3.0;
                        4.0, 5.0, 6.0];
        assert_matrix_eq!(x, x, comp = abs, tol = 1e-10);
    }

    #[test]
    #[should_panic]
    pub fn matrix_eq_absolute_mismatched_dimensions() {
        let x = matrix![1, 2, 3;
                        4, 5, 6];
        let y = matrix![1, 2;
                        3, 4];
        assert_matrix_eq!(x, y, comp = abs, tol = 0);
    }

    #[test]
    #[should_panic]
    pub fn matrix_eq_absolute_mismatched_floating_point_elements() {
        let x = matrix![1.00,  2.00,  3.00;
                        4.00,  5.00,  6.00];
        let y = matrix![1.00,  2.01,  3.00;
                        3.99,  5.00,  6.00];
        assert_matrix_eq!(x, y, comp = abs, tol = 1e-10);
    }

    #[test]
    pub fn matrix_eq_exact_compare_self_for_integer() {
        let x = matrix![1, 2, 3;
                        4, 5, 6];
        assert_matrix_eq!(x, x, comp = exact);
    }

    #[test]
    pub fn matrix_eq_exact_compare_self_for_floating_point() {
        let x = matrix![1.0, 2.0, 3.0;
                        4.0, 5.0, 6.0];
        assert_matrix_eq!(x, x, comp = exact);
    }

    #[test]
    pub fn matrix_eq_ulp_compare_self() {
        let x = matrix![1.0, 2.0, 3.0;
                        4.0, 5.0, 6.0];
        assert_matrix_eq!(x, x, comp = ulp, tol = 0);
    }

    #[test]
    pub fn matrix_eq_default_compare_self_for_floating_point() {
        let x = matrix![1.0, 2.0, 3.0;
                        4.0, 5.0, 6.0];
        assert_matrix_eq!(x, x);
    }

    #[test]
    pub fn matrix_eq_default_compare_self_for_integer() {
        let x = matrix![1, 2, 3;
                        4, 5, 6];
        assert_matrix_eq!(x, x);
    }

    #[test]
    #[should_panic]
    pub fn matrix_eq_ulp_different_signs() {
        let x = matrix![1.0, 2.0, 3.0;
                        4.0, 5.0, 6.0];
        let y = matrix![1.0, 2.0, -3.0;
                        4.0, 5.0, 6.0];
        assert_matrix_eq!(x, y, comp = ulp, tol = 0);
    }

    #[test]
    #[should_panic]
    pub fn matrix_eq_ulp_nan() {
        use std::f64;
        let x = matrix![1.0, 2.0, 3.0;
                        4.0, 5.0, 6.0];
        let y = matrix![1.0, 2.0, f64::NAN;
                        4.0, 5.0, 6.0];
        assert_matrix_eq!(x, y, comp = ulp, tol = 0);
    }

    #[test]
    pub fn matrix_eq_float_compare_self() {
        let x = matrix![1.0, 2.0, 3.0;
                        4.0, 5.0, 6.0];
        assert_matrix_eq!(x, x, comp = float);
    }

    #[test]
    pub fn matrix_eq_float_compare_self_with_eps() {
        let x = matrix![1.0, 2.0, 3.0;
                        4.0, 5.0, 6.0];
        assert_matrix_eq!(x, x, comp = float, eps = 1e-6);
    }

    #[test]
    pub fn matrix_eq_float_compare_self_with_ulp() {
        let x = matrix![1.0, 2.0, 3.0;
                        4.0, 5.0, 6.0];
        assert_matrix_eq!(x, x, comp = float, ulp = 12);
    }

    #[test]
    pub fn matrix_eq_float_compare_self_with_eps_and_ulp() {
        let x = matrix![1.0, 2.0, 3.0;
                        4.0, 5.0, 6.0];
        assert_matrix_eq!(x, x, comp = float, eps = 1e-6, ulp = 12);
        assert_matrix_eq!(x, x, comp = float, ulp = 12, eps = 1e-6);
    }

    #[test]
    pub fn matrix_eq_pass_by_ref()
    {
        let x = matrix![0.0f64];

        // Exercise all the macro definitions and make sure that we are able to call it
        // when the arguments are references.
        assert_matrix_eq!(&x, &x);
        assert_matrix_eq!(&x, &x, comp = exact);
        assert_matrix_eq!(&x, &x, comp = abs, tol = 0.0);
        assert_matrix_eq!(&x, &x, comp = ulp, tol = 0);
        assert_matrix_eq!(&x, &x, comp = float);
        assert_matrix_eq!(&x, &x, comp = float, eps = 0.0, ulp = 0);
    }

    quickcheck! {
        fn property_elementwise_vector_comparison_incompatible_vectors_yields_dimension_mismatch(
            m: usize,
            n: usize) -> TestResult {
            if m == n {
                return TestResult::discard()
            }

            // It does not actually matter which comparator we use here, but we need to pick one
            let comp = ExactElementwiseComparator;
            let ref x = Vector::new(vec![0; m]);
            let ref y = Vector::new(vec![0; n]);

            let expected = VectorComparisonResult::MismatchedDimensions { dim_x: m, dim_y: n };

            TestResult::from_bool(elementwise_vector_comparison(x.data(), y.data(), comp) == expected)
        }
    }

    quickcheck! {
        fn property_elementwise_vector_comparison_vector_matches_self(m: usize) -> bool {
            let comp = ExactElementwiseComparator;
            let ref x = Vector::new(vec![0; m]);

            elementwise_vector_comparison(x.data(), x.data(), comp) == VectorComparisonResult::Match
        }
    }

    #[test]
    fn elementwise_vector_comparison_reports_correct_mismatches() {
        use super::VectorComparisonResult::MismatchedElements;
        use super::VectorElementComparisonFailure;

        let comp = ExactElementwiseComparator;

        {
            // Single element vectors
            let x = Vector::new(vec![1]);
            let y = Vector::new(vec![2]);

            let expected = MismatchedElements {
                comparator: comp,
                mismatches: vec![VectorElementComparisonFailure {
                    x: 1, y: 2,
                    error: ExactError,
                    index: 0
                }]
            };

            assert_eq!(elementwise_vector_comparison(x.data(), y.data(), comp), expected);
        }

        {
            // Mismatch for first and last elements of a vector
            let x = Vector::new(vec![0, 1, 2]);
            let y = Vector::new(vec![1, 1, 3]);
            let mismatches = vec![
                VectorElementComparisonFailure {
                    x: 0, y: 1,
                    error: ExactError,
                    index: 0
                },
                VectorElementComparisonFailure {
                    x: 2, y: 3,
                    error: ExactError,
                    index: 2
                }
            ];

            let expected = MismatchedElements {
                comparator: comp,
                mismatches: mismatches
            };

            assert_eq!(elementwise_vector_comparison(x.data(), y.data(), comp), expected);
        }

        {
            // Check some arbitrary elements
            let x = Vector::new(vec![0, 1, 2, 3, 4, 5]);
            let y = Vector::new(vec![0, 2, 2, 3, 5, 5]);

            let mismatches = vec![
                VectorElementComparisonFailure {
                    x: 1, y: 2,
                    error: ExactError,
                    index: 1
                },
                VectorElementComparisonFailure {
                    x: 4, y: 5,
                    error: ExactError,
                    index: 4
                }
            ];

            let expected = MismatchedElements {
                comparator: comp,
                mismatches: mismatches
            };

            assert_eq!(elementwise_vector_comparison(x.data(), y.data(), comp), expected);
        }
    }

    #[test]
    pub fn vector_eq_default_compare_self_for_integer() {
        let x = Vector::new(vec![1, 2, 3 , 4]);
        assert_vector_eq!(x, x);
    }

    #[test]
    pub fn vector_eq_default_compare_self_for_floating_point() {
        let x = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);
        assert_vector_eq!(x, x);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_default_mismatched_elements() {
        let x = Vector::new(vec![1, 2, 3, 4]);
        let y = Vector::new(vec![1, 2, 4, 4]);
        assert_vector_eq!(x, y);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_default_mismatched_dimensions() {
        let x = Vector::new(vec![1, 2, 3, 4]);
        let y = Vector::new(vec![1, 2, 3]);
        assert_vector_eq!(x, y);
    }

    #[test]
    pub fn vector_eq_exact_compare_self_for_integer() {
        let x = Vector::new(vec![1, 2, 3, 4]);
        assert_vector_eq!(x, x, comp = exact);
    }

    #[test]
    pub fn vector_eq_exact_compare_self_for_floating_point() {
        let x = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);
        assert_vector_eq!(x, x, comp = exact);;
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_exact_mismatched_elements() {
        let x = Vector::new(vec![1, 2, 3, 4]);
        let y = Vector::new(vec![1, 2, 4, 4]);
        assert_vector_eq!(x, y, comp = exact);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_exact_mismatched_dimensions() {
        let x = Vector::new(vec![1, 2, 3, 4]);
        let y = Vector::new(vec![1, 2, 3]);
        assert_vector_eq!(x, y, comp = exact);
    }

    #[test]
    pub fn vector_eq_abs_compare_self_for_integer() {
        let x = Vector::new(vec![1, 2, 3, 4]);
        assert_vector_eq!(x, x, comp = abs, tol = 1);
    }

    #[test]
    pub fn vector_eq_abs_compare_self_for_floating_point() {
        let x = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);
        assert_vector_eq!(x, x, comp = abs, tol = 1e-8);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_abs_mismatched_elements() {
        let x = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);
        let y = Vector::new(vec![1.0, 2.0, 4.0, 4.0]);
        assert_vector_eq!(x, y, comp = abs, tol = 1e-8);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_abs_mismatched_dimensions() {
        let x = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);
        let y = Vector::new(vec![1.0, 2.0, 4.0]);
        assert_vector_eq!(x, y, comp = abs, tol = 1e-8);
    }

    #[test]
    pub fn vector_eq_ulp_compare_self() {
        let x = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);
        assert_vector_eq!(x, x, comp = ulp, tol = 1);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_ulp_mismatched_elements() {
        let x = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);
        let y = Vector::new(vec![1.0, 2.0, 4.0, 4.0]);
        assert_vector_eq!(x, y, comp = ulp, tol = 4);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_ulp_mismatched_dimensions() {
        let x = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);
        let y = Vector::new(vec![1.0, 2.0, 4.0]);
        assert_vector_eq!(x, y, comp = ulp, tol = 4);
    }

    #[test]
    pub fn vector_eq_float_compare_self() {
        let x = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);
        assert_vector_eq!(x, x, comp = ulp, tol = 1);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_float_mismatched_elements() {
        let x = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);
        let y = Vector::new(vec![1.0, 2.0, 4.0, 4.0]);
        assert_vector_eq!(x, y, comp = float);
    }

    #[test]
    #[should_panic]
    pub fn vector_eq_float_mismatched_dimensions() {
        let x = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);
        let y = Vector::new(vec![1.0, 2.0, 4.0]);
        assert_vector_eq!(x, y, comp = float);
    }

    #[test]
    pub fn vector_eq_float_compare_self_with_eps() {
        let x = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);
        assert_vector_eq!(x, x, comp = float, eps = 1e-6);
    }

    #[test]
    pub fn vector_eq_float_compare_self_with_ulp() {
        let x = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);
        assert_vector_eq!(x, x, comp = float, ulp = 12);
    }

    #[test]
    pub fn vector_eq_float_compare_self_with_eps_and_ulp() {
        let x = Vector::new(vec![1.0, 2.0, 3.0, 4.0]);
        assert_vector_eq!(x, x, comp = float, eps = 1e-6, ulp = 12);
        assert_vector_eq!(x, x, comp = float, ulp = 12, eps = 1e-6);
    }

    #[test]
    pub fn vector_eq_pass_by_ref()
    {
        let x = Vector::new(vec![0.0]);

        // Exercise all the macro definitions and make sure that we are able to call it
        // when the arguments are references.
        assert_vector_eq!(&x, &x);
        assert_vector_eq!(&x, &x, comp = exact);
        assert_vector_eq!(&x, &x, comp = abs, tol = 0.0);
        assert_vector_eq!(&x, &x, comp = ulp, tol = 0);
        assert_vector_eq!(&x, &x, comp = float);
        assert_vector_eq!(&x, &x, comp = float, eps = 0.0, ulp = 0);
    }
}
