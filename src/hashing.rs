// ISC License
//
// Copyright (c) 2026 Stephen Seo
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
// REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
// AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
// INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
// LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
// OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
// PERFORMANCE OF THIS SOFTWARE.

use hmac_sha512::Hash;

pub fn interactive_gen_hash_salt() -> Result<(), String> {
    let password: String = rpassword::prompt_password("Enter your password to hash: ")
        .map_err(|e| format!("Failed to get password from user: {}", e))?;

    if password.is_empty() {
        eprintln!("ERROR: password is empty!");
        return Err(String::from("Password is empty!"));
    }

    let mut hash_buf = [0u8; 64];

    getrandom::fill(&mut hash_buf).expect("Should be able to fetch random data!");

    let mut hasher = Hash::new();
    hasher.update(password.as_bytes());
    hasher.update(hash_buf);
    let hash = hasher.finalize();

    println!("hash_password = {}", hex::encode(hash));
    println!("hash_salt = {}", hex::encode(hash_buf));

    Ok(())
}
