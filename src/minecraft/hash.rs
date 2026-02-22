// https://gist.github.com/roccodev/8fa130f1946f89702f799f89b8469bc9?permalink_comment_id=4545416#gistcomment-4545416

pub fn notchian_digest(mut hash: [u8; 20]) -> String {
    let negative = (hash[0] & 0x80) == 0x80;
    let mut hex = String::with_capacity(40 + negative as usize);
    if negative {
        hex.push('-');

        // two's complement
        let mut carry = true;
        for b in hash.iter_mut().rev() {
            (*b, carry) = (!*b).overflowing_add(carry as u8);
        }
    }
    hex.extend(
        hash.into_iter()
            // extract hex digits
            .flat_map(|x| [x >> 4, x & 0xf])
            // skip leading zeroes
            .skip_while(|&x| x == 0)
            .map(|x| char::from_digit(x as u32, 16).expect("x is always valid base16")),
    );
    hex
}
