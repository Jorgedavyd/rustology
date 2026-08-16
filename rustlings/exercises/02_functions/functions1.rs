// TODO: Add some function with the name `call_me` without arguments or a return value.

fn main() {
    'b: {
        // 'b
        let mut x = 1;
        let x_mut /* 'b (covariant in T) */ = &mut x;
        'a: {
            let mut y = 1;
            let y_mut /* 'a */ = &mut y;
            call_me(x_mut, y_mut); // Don't change this line
        }
    }
}

fn call_me<'a>(x: &'a mut u8, y: &'a mut u8) -> &'a mut u8 {
    *x = 8;
    *y = 10;
    x
}
