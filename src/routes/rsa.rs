use actix_web::HttpResponse;
use num_bigint::{BigInt, BigUint, RandBigInt, Sign::Plus, ToBigUint};
use num_traits::{One, ToPrimitive, Zero};
use std::time::Instant;
//extern crate rayon;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
//use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::*;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc::channel, Arc};

fn store_keys_in_files(pub_key: &PublicKey, priv_key: &PrivateKey) {
    let mut file = File::create("public_key.txt").unwrap();
    write!(file, "{},{}", pub_key.n, pub_key.e).unwrap();

    let mut file = File::create("private_key.txt").unwrap();
    write!(file, "{},{}", priv_key.n, priv_key.d).unwrap();
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicKey {
    pub e: String,
    pub n: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateKey {
    d: String,
    n: String,
}

impl PublicKey {
    pub fn new(e: BigUint, n: BigUint) -> PublicKey {
        PublicKey {
            e: e.to_str_radix(10),
            n: n.to_str_radix(10),
        }
    }

    pub fn e(&self) -> BigUint {
        BigUint::parse_bytes(self.e.as_bytes(), 10).unwrap()
    }

    pub fn _n(&self) -> BigUint {
        BigUint::parse_bytes(self.n.as_bytes(), 10).unwrap()
    }
    pub fn _to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(Error::from)
    }

    pub fn _from_bytes(bytes: &[u8]) -> Result<PublicKey> {
        // Filter out zero bytes
        let trimmed: Vec<u8> = bytes.iter().cloned().filter(|&x| x != 0).collect();

        // Parse JSON from trimmed bytes
        serde_json::from_slice(&trimmed).map_err(Error::from)
    }
}

impl PrivateKey {
    pub fn new(d: BigUint, n: BigUint) -> PrivateKey {
        PrivateKey {
            d: d.to_str_radix(10),
            n: n.to_str_radix(10),
        }
    }

    pub fn _d(&self) -> BigUint {
        BigUint::parse_bytes(self.d.as_bytes(), 10).unwrap()
    }

    pub fn _n(&self) -> BigUint {
        BigUint::parse_bytes(self.n.as_bytes(), 10).unwrap()
    }
    pub fn _to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(Error::from)
    }

    pub fn _from_bytes(bytes: &[u8]) -> Result<PrivateKey> {
        // Filter out zero bytes
        let trimmed: Vec<u8> = bytes.iter().cloned().filter(|&x| x != 0).collect();

        // Parse JSON from trimmed bytes
        serde_json::from_slice(&trimmed).map_err(Error::from)
    }
}

fn create_public_key(p: &BigUint, q: &BigUint) -> PublicKey {
    let e = 65537u64.to_biguint().unwrap();
    let n = p * q;
    PublicKey::new(e, n)
}

fn create_private_key(p: &BigUint, q: &BigUint, pub_key: &PublicKey) -> PrivateKey {
    let one: BigUint = One::one();
    let phi = (p.clone() - &one) * (q.clone() - &one);
    let d = mod_inverse(
        &BigInt::from_biguint(Plus, pub_key.e().clone()),
        &BigInt::from_biguint(Plus, phi.clone()),
    );
    let n = p * q;
    PrivateKey::new(d.to_biguint().unwrap(), n)
}

fn extended_gcd(a: &BigInt, b: &BigInt) -> (BigInt, BigInt, BigInt) {
    if *a == BigInt::zero() {
        (b.clone(), BigInt::zero(), BigInt::one())
    } else {
        let (g, x, y) = extended_gcd(&(b % a), a);
        (g, y.clone() - (b / a) * x.clone(), x)
    }
}

fn mod_inverse(a: &BigInt, modulo: &BigInt) -> BigInt {
    let (gcd, x, _) = extended_gcd(a, modulo);
    // Failure point
    assert!(gcd == BigInt::one()); // Ensure that 'a' and 'modulo' are coprime.
    ((x % modulo) + modulo) % modulo // Ensure that the result is positive.
}

// This function generates an odd random number with a specified number of bits
fn generate_odd_random_number(bits: u32) -> BigUint {
    let mut rng = rand::thread_rng();
    let mut num = rng.gen_biguint_range(
        &BigUint::from(2u128).pow(bits - 1),
        &BigUint::from(2u128).pow(bits),
    );
    // If the number is even, add 1 to make it odd
    if num.clone() % 2u128 == BigUint::from(0u128) {
        num += BigUint::from(1u128);
    }
    num
}

// Jacobi symbol is a mathematical function used in primality testing
fn jacobi_symbol(mut a: BigUint, mut n: BigUint) -> i32 {
    // The second argument should be odd
    assert!(n.clone() % 2u8 == 1u64.into());
    let mut s = 1;
    // Keep halving 'a' until it becomes odd
    while a != 0u64.into() {
        while a.clone() % 2u8 == 0u64.into() {
            a /= 2u8;
            let n_mod_8: u8 = (&n % 8u8).to_u8().unwrap();
            // If n mod 8 is 3 or 5, flip the sign of s
            if n_mod_8 == 3 || n_mod_8 == 5 {
                s = -s;
            }
        }
        // Swap 'n' and 'a'
        std::mem::swap(&mut n, &mut a);
        // If both n and a are congruent to 3 mod 4, flip the sign of s
        if (&n % 4u8).to_u8().unwrap() == 3 && (&a % 4u8).to_u8().unwrap() == 3 {
            s = -s;
        }
        a %= &n;
    }
    // If n is 1, return s; else return 0
    if n == 1u64.into() {
        s
    } else {
        0
    }
}

// This function does modular exponentiation
pub fn mod_exp(mut base: BigUint, mut exponent: BigUint, modulus: BigUint) -> BigUint {
    let mut result: BigUint = if exponent.is_zero() {
        Zero::zero()
    } else {
        One::one()
    };
    base %= modulus.clone();

    // Keep squaring the base and reducing the exponent until the exponent becomes zero
    while !exponent.is_zero() {
        if &exponent % 2u8 == 1u8.into() {
            result = (result * &base) % &modulus;
        }
        base = (&base * &base) % &modulus;
        exponent >>= 1;
    }

    result
}

fn solovay_strassen(n: &BigUint, iterations: u32) -> bool {
    // 2 and 3 are prime, so we just return true for these
    if n == &BigUint::from(2u8) || n == &BigUint::from(3u8) {
        return true;
    }

    let n_minus_one = n - BigUint::one();
    let mut rng = rand::thread_rng();
    for _ in 0..iterations {
        // Generate a random number 'a' in the range [2, n-1]
        let a: BigUint =
            rng.gen_biguint_range(&BigUint::from(2u64), &BigUint::from(n.to_u64_digits()[0]));
        let a_clone = a.clone(); // Clone it as we will use 'a' multiple times
        let n_clone = n.clone(); // Similarly clone 'n'
        let x = jacobi_symbol(a_clone.clone(), n_clone.clone()); // Calculate the Jacobi symbol
        let expected_result = if x == -1 {
            n_minus_one.clone() // If Jacobi symbol is -1, we expect n-1 in the next check
        } else {
            BigUint::from(x.unsigned_abs() as u64) // Otherwise we expect the absolute value of the Jacobi symbol
        };

        // If Jacobi symbol is 0, or a^(n-1)/2 is not congruent to Jacobi symbol mod n,
        // the number is definitely composite
        if x == 0
            || mod_exp(a_clone.clone(), n_minus_one.clone() >> 1, n_clone.clone())
                != expected_result
        {
            return false;
        }
    }
    // If we passed all tests, we guess the number is probably prime
    true
}
use std::thread;

// The 'main' function kicks off the whole process
pub fn gen_keys() -> (PublicKey, PrivateKey, BigUint, BigUint, BigUint, u32) {
    let now_master = Instant::now(); // And another one to measure total time
    let num_tries = 32; // Number of random numbers we generate and check for primality
    let num_bits = 1024; // The size of the numbers we're interested in
    let num_iterations = 20; // Number of iterations in the Solovay-Strassen test

    // Find the first prime number 'p'

    let p_thread = thread::spawn(move || find_prime(num_tries, num_bits, num_iterations, None));

    let q_thread = thread::spawn(move || find_prime(num_tries, num_bits, num_iterations, None));

    let (mut p, mut odd_nums_tried_p, mut values_used_p) = p_thread.join().unwrap();
    let (mut q, mut odd_nums_tried_q, mut values_used_q) = q_thread.join().unwrap();

    // If p and q are the same, regenerate BOTH
    while p == q {
        println!("Found duplicate primes, regenerating...");
        (p, odd_nums_tried_p, values_used_p) =
            find_prime(num_tries, num_bits, num_iterations, None);
        (q, odd_nums_tried_q, values_used_q) =
            find_prime(num_tries, num_bits, num_iterations, Some(&p));
    }
    let total_odd_nums_tried = odd_nums_tried_p + odd_nums_tried_q;
    let total_values_used = values_used_p + values_used_q;

    let pub_key = create_public_key(&p, &q);
    let priv_key = create_private_key(&p, &q, &pub_key);
    // Store the keys in files
    store_keys_in_files(&pub_key, &priv_key);
    //println!("Stored keys in {}ms", now.elapsed().as_millis()); // Print the time it took
    // Print the total time it took for all operations
    println!(
        "Completed all operations in {}ms",
        now_master.elapsed().as_millis()
    );
    (
        pub_key,
        priv_key,
        p,
        q,
        total_odd_nums_tried,
        total_values_used,
    )
}

use std::sync::atomic::AtomicUsize;
fn find_prime(
    num_tries: u32,
    num_bits: u32,
    num_iterations: u32,
    exclude: Option<&BigUint>,
) -> (BigUint, BigUint, u32) {
    let (tx, rx) = channel();
    let found = Arc::new(AtomicBool::new(false));

    let odd_nums_tried = Arc::new(AtomicUsize::new(0));
    let values_used = Arc::new(AtomicUsize::new(0));

    loop {
        (0..num_tries).into_par_iter().for_each_with(
            (
                tx.clone(),
                found.clone(),
                odd_nums_tried.clone(),
                values_used.clone(),
            ),
            move |(s, found, odd_nums_tried, values_used), _| {
                // check if a prime has already been found
                if found.load(Ordering::Relaxed) {
                    return;
                }

                let possible_prime = generate_odd_random_number(num_bits);
                odd_nums_tried.fetch_add(1, Ordering::Relaxed);
                if exclude.map_or(true, |x| *x != possible_prime) {
                    let is_prime = solovay_strassen(&possible_prime, num_iterations);
                    if is_prime {
                        // If a prime is found, send it and update the found flag
                        values_used.fetch_add(num_iterations as usize, Ordering::Relaxed);
                        s.send(possible_prime).unwrap();
                        found.store(true, Ordering::Relaxed);
                    }
                }
            },
        );

        // If a prime is found, receive it from the channel and return it with the counters
        if let Ok(prime) = rx.try_recv() {
            return (
                prime,
                BigUint::from(odd_nums_tried.load(Ordering::Relaxed) as u64),
                values_used.load(Ordering::Relaxed) as u32,
            );
        }

        // Reset the found flag
        found.store(false, Ordering::Relaxed);
    }
}

fn print_statistics(
    odd_nums_tried: &BigUint,
    values_used: u32,
    iterations: u32,
) -> (BigUint, u32, f64) {
    let confidence: f64 = 1.0 - 1.0 / (4.0f64.powi(iterations as i32));
    (odd_nums_tried.clone(), values_used, confidence)
}

pub async fn generate_rsa_keys() -> HttpResponse {
    let (_pub_key, _priv_key, p, q, odd_nums_tried, values_used) = gen_keys();
    let (_odd_nums_tried, _values_used, confidence) =
        print_statistics(&odd_nums_tried, values_used, 20); // 55 is the number of iterations, as in your code
    println!("{}", confidence);

    HttpResponse::Ok().json(json!({
        "prime1": p.to_str_radix(10),
        "prime2": q.to_str_radix(10),
        "oddNumbersTried": odd_nums_tried.to_str_radix(10),
        "valuesUsed": values_used,
        "confidence": confidence,
    }))
}
