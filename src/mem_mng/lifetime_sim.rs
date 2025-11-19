#[allow(unused_variables)]

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
    fn it_doesnt_work() {
        let mut x = "hello world";
        //          strtok_error(&'a mut &'a str, ' '); ->
        //          strtok_error(&'a mut &'static str, ' '); ->
        //          given the strtok_err<'a> generic 'a lifetime, it tries to coerce the
        //          first argument for any 'a into &'a mut &'a str.
        //          Given that we're giving a &'x mut &'static str, let's look at it from
        //          two perspectives:
        //          1. Let's say it is coerced into the stricter type ('static), it would be
        //             undefined behavior.
        //          2. Let's say it is coerced into the weaker type ('x, that would be covariance), then you would be
        //          trying to change &'x mut &'static str into &'x mut &'x str but that is illegal
        //          given that &'a mut T is invariant in T
        //          but you cannot coerce the 'static lifetime to behave like 'a lifeitme
        //          in this case given that &'a mut T is invariant in T
        //          That's why the solution is to add a lifetime, because in other case, you would
        //          be violating the rules of variance directly by trying to coerce &'x mut
        //          &'static str into &'x mut &'x str while for<'a> &'a mut T is invariant in T
        let hello = strtok_error(&mut x, ' ');
        assert_eq!(hello, "hello");
        assert_eq!(x, "world");
    }

    #[test]
    fn it_works() {
        let mut x = "hello world";
        let hello = strtok_ok(&mut x, ' ');
        assert_eq!(hello, "hello");
        assert_eq!(x, "world");
    }
}

fn main() {
    let eph_str = String::new();
    let static_str: &'static str = "hello world";
    let mut a_str /* &'a str */ = &*eph_str;
    a_str = static_str; // &'static -> &'a
}

// Subtyping
// T        <: U  if T is at least as useful as U
// 'static  <: 'a if T is at least as useful as U
//
// class Animal;
// class Cat: Animal;
//
// Cat <: Animal kind of
//
//
// Covariance
// Given T and U such that T <: U F<T> is covariant in T if F<T> <: F<U>
//
// &'static str <: &'a str, therefore F: &'a str is covariant in 'a
//
// Contravariance
//
// Given T and U such that T <: U F<T> is contravariant in T if F<U> <: F<T>
//
// fn(&'a str) -> () <: fn (&'static str) -> ()
// you can't replace the first one with the latter because the latter narrows the
// domain of lifetimes that the function can take. You basically cannot use both functions
// with the same input &'a given that fn (&'static str) is specting to get a lifetime of at least
// 'static. That's where the concept of "usefulness" arises. The usefulness would be defined as,
// given that I would need to use some &'a T in this function, is the other function enabling the
// domain of lifetimes plus more lifetimes? if not and it is narrowing the domain, then it should
// be contravariant given that we are expecting the function to be useful for some &'a out of the
// domain of &'static requirement
//
// Invariance
// Given T and U, even though T <: U we cannot assume nothing from subtyping of F<T> and F<U>
//
// e.g.
//
// fn foo(s: &mut &'a str, a: &'a str) {
//      *s = a;
// }
//
// fn main() {
//      let mut static_str: &'static str = "hello world";
//      let y = String::new();
//      let eph_str = &*y;
//      // Let's imagine &'a mut T is covariant in T
//      // Then given that 'static <; 'a it would look like
//      // this:
//      foo(&   mut static_str, eph_str);
//      // (&'_ mut &'a str,    &'a str)
//      // given that 'static "could be downgraded safely"
//      // but looking closely to the definition of foo,
//      // it is modifying the content of x ('static) with some
//      // other reference with a shorter lifetime, so any attempt
//      // of accessing that memory after 'a lifetime would be undefined behavior,
//      // and given that 'static implies that it can be used throughout the whole
//      // program, any use outside of the 'a scope is invalid, therefore, by negation
//      // &'a mut T cannot be covariant in T
//      // This is an example of invariance in T, but it can logically be covariant in 'a
//      // given that naturally you could have some 'b: 'a such that 'b <: 'a, and
//      // &'b mut T <: &'a mut T could be as well &'b U <: &'a U, which is covariant,
//      // but more importantly, the lifetime of the mutable reference is not tied to the usability
//      // of it as we discussed to demonstrate that &'a mut T is invariant in T, messing with the
//      // inner lifetime of the thing that is being referenced to, on the othre hand, whether
//      // the actual reference lives long enough goes into the same box as the classic &'b T <:
//      &'a T if 'b: 'a
//      drop(y);
//      println("{}", x); (x holds a reference to y which is dropped but x stands on its 'static
//      definition, therefore, UB)
// }
//
