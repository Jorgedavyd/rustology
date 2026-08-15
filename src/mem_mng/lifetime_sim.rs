#![allow(unused_variables, unused_assignments)]
//! ## This must NOT compile:
//! ```compile_fail
//! pub fn strtok_error<'a>(s: &'a mut &'a str, delim: char) -> &'a str {
//!     if let Some(index) = s.find(delim) {
//!         let prefix: &str = &s[..index];
//!         let suffix: &str = &s[(index + delim.len_utf8())..];
//!         *s = suffix;
//!         prefix
//!     } else {
//!         let prefix: &str = *s;
//!         *s = "";
//!         prefix
//!     }
//! }
//!
//! #[test]
//! fn it_doesnt_work() {
//!     let mut x = "hello world";
//!     let hello = strtok_error(&mut x, ' ');
//!     assert_eq!(hello, "hello");
//!     assert_eq!(x, "world");
//! }
//! ```
//!
//! It fails because:
//! - `'a` is used for **both** the outer mutable ref and the inner `&str`
//! - This forces **`&'a mut &'a str`**, which is **invariant in `'a`**
//! - Therefore `&mut &'static str` **cannot be coerced into** `&mut &'a str`
//! - This violates **invariance**, and Rust rejects it.
//!
//! ## This version compiles (`'_` breaks the tie):
//! ```rust
//! pub fn strtok_ok<'a>(s: &'_ mut &'a str, delim: char) -> &'a str {
//!     if let Some(index) = s.find(delim) {
//!         let prefix: &str = &s[..index];
//!         let suffix: &str = &s[(index + delim.len_utf8())..];
//!         *s = suffix;
//!         prefix
//!     } else {
//!         let prefix: &str = *s;
//!         *s = "";
//!         prefix
//!     }
//! }
//! ```

// ----------------- ACTUAL CODE BELOW ----------------------

pub fn strtok_error<'a>(s: &'a mut &'a str, delim: char) -> &'a str {
    if let Some(index) = s.find(delim) {
        let prefix: &str = &s[..index];
        let suffix: &str = &s[(index + delim.len_utf8())..];
        *s = suffix;
        prefix
    } else {
        let prefix: &str = *s;
        *s = "";
        prefix
    }
}

pub fn strtok_ok<'a>(s: &'_ mut &'a str, delim: char) -> &'a str {
    if let Some(index) = s.find(delim) {
        let prefix: &str = &s[..index];
        let suffix: &str = &s[(index + delim.len_utf8())..];
        *s = suffix;
        prefix
    } else {
        let prefix: &str = *s;
        *s = "";
        prefix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut x = "hello world";
        let hello = strtok_ok(&mut x, ' ');
        assert_eq!(hello, "hello");
        assert_eq!(x, "world");
    }

    /// ```compile_fail
    /// use super::strtok_error;
    ///
    /// #[test]
    /// fn it_doesnt_work() {
    ///     let mut x = "hello world";
    ///     let hello = strtok_error(&mut x, ' ');
    ///     assert_eq!(hello, "hello");
    ///     assert_eq!(x, "world");
    /// }
    /// ```
    ///
    /// This fails due to **invariance** of `&mut T`.
    /// `'static` cannot be coerced into `'a`.
    ///
    #[test]
    fn compile_fail_example_is_documented() {
        // This test only exists to hold the doc test.
        assert!(true);
    }
}

fn main() {
    let eph_str = String::new();
    let static_str: &'static str = "hello world";
    let mut a_str /* &'a str */ = &*eph_str;
    a_str = static_str; // &'static -> &'a
}
