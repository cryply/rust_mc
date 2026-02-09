# RSA: A Milestone in Cryptography

RSA was first publicly described by Ron Rivest, Adi Shamir, and Leonard Adleman (MIT) in 1977. Their work built heavily on the idea of an asymmetric public-private key cryptosystem published in 1976 by Whitfield Diffie and Martin Hellman (Stanford).

*Interestingly, British intelligence agency GCHQ had independently discovered an equivalent algorithm in 1973 (by Clifford Cocks), but it remained classified until 1997.*

RSA became the first practical public-key encryption system and remains widely used today. Here I provide a simple explanation of the cipher — more as a note to my future self than a claim of expertise.

## Key Generation

**Step 1.** Choose two large prime numbers $p$ and $q$.

**Step 2.** Calculate the modulus: $n = p \times q$

**Step 3.** Calculate Euler's totient function $\varphi(n)$.

This function counts positive integers less than $n$ that are relatively prime (co-prime) to $n$.

> **What does co-prime mean?**
> 
> Two numbers are **co-prime** (or **relatively prime**) if their greatest common divisor (GCD) equals 1 — meaning they share no common factors other than 1.
> 
> Examples:
> - 8 and 15 are co-prime: factors of 8 are {1, 2, 4, 8}, factors of 15 are {1, 3, 5, 15} → GCD = 1 ✓
> - 8 and 12 are **not** co-prime: both divisible by 4 → GCD = 4 ✗
> - Any prime number is co-prime with all numbers except its multiples

For example: $\varphi(15) = \varphi(3 \times 5) = (3-1)(5-1) = 8$

The 8 co-prime numbers to 15 are: {1, 2, 4, 7, 8, 11, 13, 14}

Since $p$ and $q$ are prime, the formula simplifies to:

$$\varphi(n) = (p-1)(q-1)$$

> **Important:** $\varphi(n)$ must be kept private.

**Step 4.** Choose public exponent $e$ such that:
- $1 < e < \varphi(n)$
- $\gcd(e, \varphi(n)) = 1$ (i.e., $e$ and $\varphi(n)$ are co-prime)

Common choices: 3, 17, or 65537 (65537 = $2^{16} + 1$ is most popular due to efficient computation).

**Step 5.** Calculate private exponent $d$ using the Extended Euclidean Algorithm (EEA).

We need $d$ such that:

$$d \times e \equiv 1 \pmod{\varphi(n)}$$

**Why EEA works:**

The Extended Euclidean Algorithm finds $x$ and $y$ such that:

$$a \cdot x + b \cdot y = \gcd(a, b)$$

Let $a = e$ and $b = \varphi(n)$. Since they are co-prime by definition, $\gcd(e, \varphi(n)) = 1$:

$$e \cdot x + \varphi(n) \cdot y = 1$$

Taking modulo $\varphi(n)$ on both sides:

$$e \cdot x \equiv 1 \pmod{\varphi(n)}$$

This $x$ is our private key $d$.

**Example:** If $\varphi(n) = 3120$ and $e = 17$, then EEA gives $d = 2753$

Verification: $2753 \times 17 \mod 3120 = 46801 \mod 3120 = 1$ ✓

## The Keys

| Key | Components | Visibility |
|-----|------------|------------|
| **Public Key** | $(e, n)$ | Shared openly |
| **Private Key** | $(d, n)$ | Kept secret |

## Security Foundation

The security of RSA relies on the **integer factorization problem**: given a large $n$, it is computationally infeasible to find its prime factors $p$ and $q$.

- Without $p$ and $q$, you cannot compute $\varphi(n)$
- Without $\varphi(n)$, you cannot derive $d$ from $e$
- Therefore, knowing only the public key $(e, n)$, the private key $d$ remains secret

## Encryption and Decryption

**Encryption** (using recipient's public key):

$$c = m^e \mod n$$

where $m$ is the message and $m < n$.

**Decryption** (using private key):

$$m = c^d \mod n$$

**Why it works:**

$$c^d = (m^e)^d = m^{ed} \pmod{n}$$

Since $ed \equiv 1 \pmod{\varphi(n)}$, we have $ed = 1 + k \cdot \varphi(n)$ for some integer $k$.

By Euler's theorem, $m^{\varphi(n)} \equiv 1 \pmod{n}$ (when $\gcd(m, n) = 1$):

$$m^{ed} = m^{1 + k \cdot \varphi(n)} = m \cdot (m^{\varphi(n)})^k \equiv m \cdot 1^k = m \pmod{n}$$

## Practical Considerations

**Message size limitation:** RSA can only encrypt messages where $m < n$. For large data, hybrid encryption schemes are used — RSA encrypts a symmetric key, which then encrypts the actual data using fast ciphers like AES.

**Digital signatures:** RSA is also widely used for digital signatures:
- **Signing:** $s = m^d \mod n$ (using private key)
- **Verification:** $m = s^e \mod n$ (using public key)

**Key sizes:** Modern RSA implementations use 2048-bit or 4096-bit keys. Smaller keys (512-bit, 1024-bit) are considered insecure.

## Summary

```
Key Generation:
  1. Choose primes p, q
  2. Compute n = p × q
  3. Compute φ(n) = (p-1)(q-1)
  4. Choose e: 1 < e < φ(n), gcd(e, φ(n)) = 1
  5. Compute d: d × e ≡ 1 (mod φ(n))

Public key:  (e, n)
Private key: (d, n)

Encrypt: c = mᵉ mod n
Decrypt: m = cᵈ mod n
```