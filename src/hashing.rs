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
