//! Linear interpolation / extrapolation

extern crate num_traits;

use std::iter;
use std::iter::{Iterator, Skip, Chain, Once};
use std::ops::{Add, Sub, Mul};
use num_traits::Float;

/// Types which are amenable to linear inter/extrapolation.
///
/// This is mainly intended to be useful for complex
/// numbers, vectors, and other types which may be multiplied by a
/// scalar while retaining their own type.
pub trait Lerp<F> {
    /// Interpolate / extrapolate between `self` and `other` using `t` as the parameter.
    ///
    /// At `t == 0.0`, the result is equal to `self`.
    /// At `t == 1.0`, the result is equal to `other`.
    /// At all other points, the result is a mix of `self` and `other`, proportional to `t`.
    ///
    /// ```
    /// # use julia_set::lerp::Lerp;
    /// let four_32 = 3.0_f32.lerp(5.0, 0.5);
    /// assert_eq!(four_32, 4.0);
    /// let four_64 = 3.0_f64.lerp(5.0, 0.5);
    /// assert_eq!(four_64, 4.0);
    /// ```
    fn lerp(self, other: Self, t: F) -> Self;
}

/// Types which can construct a lerping iterator from one point to another
/// over a set number of steps.
///
/// For the most part, this is automatically implemented.
pub trait LerpIter {
    /// Create an iterator which lerps from `self` to `other`.
    ///
    /// The iterator is half-open: it includes `self`, but not `other`
    ///
    /// # Example
    ///
    /// ```
    /// # use julia_set::lerp::LerpIter;
    /// // lerp between 3 and 5, collecting two items
    /// let items: Vec<f64> = 3.0_f64.lerp_iter(5.0, 2).collect();
    /// assert_eq!(vec![3.0, 4.0], items);
    /// ```
    fn lerp_iter(self, other: Self, steps: usize) -> LerpIterator<Self> where Self: Sized;

    /// Create an iterator which lerps from `self` to `other`.
    ///
    /// The iterator is closed: it returns both `self` and `other`.
    ///
    /// Note when `steps == 1`, `other` is returned instead of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// # use julia_set::lerp::LerpIter;
    /// assert_eq!(vec![3.0, 5.0], 3.0_f64.lerp_iter_closed(5.0, 2).collect::<Vec<f64>>());
    /// ```
    fn lerp_iter_closed(self,
                        other: Self,
                        steps: usize)
                        -> Skip<Chain<LerpIterator<Self>, Once<Self>>>
        where Self: Copy,
              LerpIterator<Self>: Iterator<Item = Self>
    {
        // reduce the number of times we consume the sub-iterator,
        // because we unconditionally add an element to the end.
        if steps == 0 {
            LerpIterator::new(self, other, steps).chain(iter::once(other)).skip(1)
        } else {
            LerpIterator::new(self, other, steps - 1).chain(iter::once(other)).skip(0)
        }
    }
}

/// Default, generic implementation of Lerp.
///
/// Note that due to the implementation details, LerpIterator is only actually
/// an iterator for those types `T` which fit the constraint `Mul<f64, Output = T>`.
/// This means that though you can use the `lerp` method on f32s, it will not work to
/// iterate over the results of calling `lerp_iter` on an f32. Instead, up-cast
/// your f32 as an f64 before calling: `(example_f32 as f64).lerp_iter(...)`.
///
/// This default implementation is mainly intended to be useful for complex
/// numbers, vectors, and other types which may be multiplied by a
/// scalar while retaining their own type.
impl<T, F> Lerp<F> for T
    where T: Copy + Add<Output = T> + Sub<Output = T> + Mul<F, Output = T>,
          F: Float
{
    fn lerp(self, other: T, t: F) -> T {
        self + ((other - self) * t)
    }
}

impl<T> LerpIter for T
    where T: Lerp<f64> + Sized
{
    fn lerp_iter(self, other: T, steps: usize) -> LerpIterator<T> {
        LerpIterator::new(self, other, steps)
    }
}

/// An iterator across a range defined by its endpoints and the number of intermediate steps.
pub struct LerpIterator<T> {
    begin: T,
    end: T,
    steps: usize,
    current_step: usize,
}

impl<T> LerpIterator<T> {
    fn new(begin: T, end: T, steps: usize) -> LerpIterator<T> {
        LerpIterator {
            begin: begin,
            end: end,
            steps: steps,
            current_step: 0,
        }
    }
}

impl<T> Iterator for LerpIterator<T>
    where T: Lerp<f64> + Copy
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.current_step >= self.steps {
            None
        } else {
            let t = self.current_step as f64 / self.steps as f64;
            self.current_step += 1;
            Some(self.begin.lerp(self.end, t))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = if self.current_step >= self.steps {
            0
        } else {
            self.steps - self.current_step
        };
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for LerpIterator<T> where T: Lerp<f64> + Copy {}