# Mathematics of BBS+ Signatures and Zero-Knowledge Proofs

> [!TIP]
> **New to Elliptic Curves?**
> Before diving into the rather complex mathematics of BBS+ signatures (which I still have trouble fully grasping sometimes), I highly recommend not skipping the introduction: **[Mathematics 101: Elliptic Curves and Field Extensions](00_mathematics_101.md)**.

As I mentioned, I'm not a professional mathematician—I just know the basics. But I've tried my best to understand and explain the mathematics of the BBS+ signature scheme using bilinear pairings, and how it maps to our Rust implementation.

Mainly for curiosity, improve knowledge, the purpose of this quick demo is to understand the core of BBS+ and ZKP implementations.

In addition, pairing based bilinear group have multiple application Identity-Based Encryption _(IBE, - that dislike cause of central pooint of trust to deliver private key- )_, Zero-Knowledge Proofs(ZKP) zk-SNARKs, etc ...


## 0. Mathematical Preliminary: Finite Fields and Curves

**Step 1: The Finite Field $\mathbb{F}_{p}$ (The "Playground")**
A finite field (denoted $\mathbb{F}_{p}$ or GF(p)) is a set of numbers from $0$ to $p-1$. In this set, all operations (addition, multiplication, division) are performed modulo $p$.
These are not yet curve points, they are just scalars (coordinates).
For BLS12-381, this playground contains $p$ elements (where $p$ is your 381-bit number).

**Step 2: The Curve Group $E(\mathbb{F}_p)$**
When we set the equation $y^2 = x^3 + 4$ over this finite field, we gather all pairs of coordinates $(x, y)$ belonging to $\mathbb{F}_{p}$ that satisfy the equality. The set of these points (plus a special point called the "point at infinity", denoted $\mathcal{O}$ or 0).
In this group, the operation is not multiplication, it is point addition ($P + Q = R$).
The total number of points present on this curve is called the Order of the curve (denoted $N$).

**Step 3: Why extract the subgroup $G_{1}$ of order $r$?**

The curve $E(\mathbb{F}_p)$ defined by the equation $y^2 = x^3 + 4$ over $\mathbb{F}_p$ does not have a prime order $N$ (it's a composite number).
To be secure it must be performed in a prime-order subgroup (of size $r$). 

One reason is to prevent **small subgroup attacks**, if an attacker forces you to compute using a point belonging to these small subgroups, they can easily piece together and guess your private key in just a few steps (via the Pohlig-Hellman algorithm).

May have other reasons ? 

**Step 4: Understanding the cofactor $h_{1}$ (The filter)**

Since the total order of the curve is $N$ and the size of our target subgroup is $r$, the cofactor $h_{1}$ is simply the ratio between the two:

$$ h_{1} = \frac{N}{r} \implies N = h_{1} \cdot r $$

The mathematical proof of the effect of $h_{1}$:
If you choose a point $P_{random}$ completely at random on the curve, it belongs to $E(\mathbb{F}_p)$. By definition of the order of a group, if we multiply this point by the total order $N$, it cancels out:

$$ N \cdot P_{random} = \mathcal{O} $$

Let's replace $N$ with its formula ($h_1 \cdot r$):

$$ (r \cdot h_{1}) \cdot P_{random} = \mathcal{O} $$
$$ r \cdot (h_{1} \cdot P_{random}) = \mathcal{O} $$

Look closely at the expression in parentheses: $(h_1 \cdot P_{random})$. It is a new point. Let's call it $P_{final}$.
Since $r \cdot P_{final} = \mathcal{O}$, this provides the absolute mathematical proof that $P_{final}$ possesses the exclusive property of the elements of the subgroup $G_{1}$. Multiplying by $h_{1}$ acts as a filter that eliminates all undesirable components of the point to project it into $G_{1}$.


## 1. The Core Concept: Bilinear Pairings

Before diving into pairings, we need to understand the two different "worlds" that make up elliptic curve cryptography. Think of them as the **Algebra world** and the **Geometry world**:

* **The Scalar Field (The World of Numbers)**: This is simply a giant playground of normal numbers $\{0, 1, 2, \dots, r-1\}$ where your secret keys, passwords, and messages live. You can add, subtract, multiply, and divide them like normal math, but the results always wrap around a massive prime number \(r\) 
* **The Elliptic Curve Groups $G_{1}$ and $G_{2}$ (The World of Points)**: This is a world of geometric shapes. Instead of raw numbers, your public keys and signatures are points $\mathbf{(x, y)}$ sitting on a specific mathematical curve _(like BLS12_381)_. You **cannot** multiply or divide two points together; you can only "hop" from one point to another by adding them geometrically.

BBS+ signatures rely on **Pairing-Based Cryptography**, specifically over mathematical point groups $G_1$, $G_2$, and $G_T$ equipped with a bilinear map (a "pairing" function) denoted as $e: G_1 \times G_2 \to G_T$.

A pairing function $e$ has a unique property called **bilinearity**:

$$ e(a \cdot g_1, b \cdot g_2) = e(g_1, g_2)^{a \cdot b} $$

This property allows us to verify multiplicative relationships between hidden values stored as exponents, which is the foundational building block of Zero-Knowledge Proofs.

> It could be sumup to take a point in the curve and project in the extension that is now a polynome in the extension that can be multiplied.

in version 0.4.1 of bbs crate they use a specific curve `BLS12_381` to [generate](https://docs.rs/crate/bbs/0.4.1/source/src/lib.rs#43) the keys.


## 2. Key Generation (`Issuer::new_keys`)

To set up the system to sign $L$ messages (in our code, `DEFAULT_MESSAGE_COUNT = 5`), the Issuer generates cryptographic keys:

1.  **Public Parameters**: 
    - A base generator point $g_1 \in G_1$ and another $g_2 \in G_2$.
    - A set of generator points in $G_1$: $h_0, h_1, h_2, \dots, h_L$ (one generator for each attribute/claim we want to sign, e.g., one for the Name, one for IsMajor, etc.).
2.  **Secret Key ($SK$)**: A randomly chosen scalar number $x \in \mathbb{Z}_p$.
3.  **Public Key ($PK$)**: The value $w = x \cdot g_2 \in G_2$.

*(Note on Notation: Academic cryptography papers historically use multiplicative notation like $g^x$. However, because elliptic curves mathematically perform point addition, we are using the modern, programmatic additive notation $x \cdot g_2$ throughout this document to better match how developers and Rust libraries handle Elliptic Curve Cryptography).*

*Code Mapping:* In `issuer.rs`, the generation of these structures looks like this:
```rust
// pk is the Public Key (w), sk is the Secret Key (x)
let (pk, sk) = Issuer::new_keys(n)?; 
```

## 3. Signing Messages (`Signer::sign`)

When Alice wants the Issuer to sign her identity (e.g., Name, IsMajor, Address), the fields are first hashed into numerical scalar values: $m_1, m_2, m_3$.

The Issuer chooses two random blinding scalars $e, s \in \mathbb{Z}_p$.
The signature is a tuple $(A, e, s)$ where $A \in G_1$ is calculated as:

$$ A = \frac{1}{e + x} \cdot \left( g_1 + s \cdot h_0 + \sum_{i=1}^L m_i \cdot h_i \right) $$

**What are $h_0, h_i$ ?**
Do not confuse these with the scalar cofactor $h_1$ (the filter) we talked about in the [Mathematics 101](00_mathematics_101.md) guide! Here, $h_0, h_1, h_2...$ are simply **public generator points** on the curve in $G_1$ (just like $g_1$). However, we need to make sure these are strictly in the prime subgroup $G_1$ of order $r$.

Lets check the **`bbs` crate** [src/lib.rs](https://docs.rs/crate/bbs/0.4.1/source/src/lib.rs#43), it imported by [pairing_plus::hash_to_curve](https://docs.rs/pairing-plus/latest/src/pairing_plus/hash_to_curve.rs.html), the filter out via the cofactor are perform by the function `p.clear_h()`

**Why add the random scalars $e$ and $s$?**
Since the whole signature tuple $(A, e, s)$ is given back to Alice, $s$ is obviously not a secret kept from her. Instead, $e$ and $s$ act as **randomizers**. 
If Alice asks the Issuer to sign the exact same attributes a second time, the Issuer will pick new random $e$ and $s$. The resulting point $A$ will be completely different! This ensures that signatures are randomized and prevents anyone from tracking Alice by comparing her signatures.

We will see exactly how a Verifier can validate this fraction equation in Step 5 using bilinear pairings, without needing to know the secret key $x$!

*Code Mapping:* In `issuer.rs`, this mathematical exponentiation to compute $(A, e, s)$ is handled internally by the `Signature::new` function:
```rust
let messages_to_sign: Vec<SignatureMessage> =
    messages.iter().map(|m| SignatureMessage::hash(m)).collect();

// This computes the signature tuple (A, e, s) and compresses it into 112 bytes
let signature = Signature::new(messages_to_sign.as_slice(), &self.sk, &self.pk)?;
```

## 4. Zero-Knowledge Proof of Knowledge (PoK) (`prover.rs`)

### **Step 1: Blinding (Randomization)**

Alice wants to show her credentials to a Verifier, but she doesn't want them to track her across different visits. To stay completely anonymous, she must **blind** her original signature. 

This protocol is based on the official research paper framework:  
* **[BBS+ Applications, Standardization,
and a Bit of Theory](https://csrc.nist.gov/csrc/media/presentations/2023/crclub-2023-10-18/images-media/20231018-crypto-club--greg-and-vasilis--slides--BBS.pdf)** 

#### **What is $P$?**
Before blinding anything, need to define **$P$**, the **Base Aggregate Point** (a geometric point in the $G_1$ group) . It is a multi-scalar encapsulation combining all of Alice's messages ($m_1, m_2, \dots$), public generators ($g_1, h_0, h_1, \dots$), and the signature blinding factor ($s$) [2019/814.pdf]:

$$P = g_1 + h_0 \cdot s + h_1 \cdot m_1 + h_2 \cdot m_2 + \dots$$

This point $P$ was originally assembled by the Issuer during the signing phase. The signature ($A$) Alice holds is mathematically bound to this specific point via the equation: $(x + e) \cdot A = P$. 

#### **The Blinding Math Explained**
To hide her signature $A$ and her secret attributes, Alice picks two new random scalars, $r_1$ and $r_2$. The prover computes three blinded components:

> [!NOTE]
> The mathematics here gets pretty intense, and I still struggle to fully grasp all of it (especially the $D$ component). I did my best to dig in and map the formulas from the academic paper to their actual implementation in the `bbs` Rust crate.

* **$A' = r_1 \cdot A$ (Blinding the Signature)**  
  Alice multiplies her signature point $A$ by the random scalar $r_1$. This shifts the signature to a completely random location on the curve, turning it into untrackable random noise for the Verifier.
  * *Code Source:* Declared as `a_prime` inside [`pok_sig.rs`](https://docs.rs/crate/bbs/0.4.1/source/src/pok_sig.rs) during `PoKOfSignature::init()`.

* **$\bar{A} = r_1 \cdot P - e \cdot A'$ (Hiding the Connection)**  
  This equation protects the mathematical relationship between the signature and the data point $P$. By injecting $r_1$ and her public signature exponent $e$, she proves she knows a valid signature without revealing her original signature $A$ or the signature exponent $e$. 
  * *Code Source:* Declared as `a_bar` inside [`pok_sig.rs`](https://docs.rs/crate/bbs/0.4.1/source/src/pok_sig.rs), computed via `a_bar.sub_assign(&a_bar_denom)`.

* **$D = r_1 \cdot P - r_2 \cdot h_0$ (Attribute Commitment)**  
  To hide undisclosed messages, the base point $P$ and the public generator $h_0$ are committed into a distinct blinding point $D$ using $r_1$ and $-r_2$. This value acts as a secure anchor for the hidden multi-message structure.
  * *Code Source:* Declared as lowercase `d` inside [`pok_sig.rs`](https://docs.rs/crate/bbs/0.4.1/source/src/pok_sig.rs), built via `CommitmentBuilder::new()`.

As a user of the crate, all this underlying logic is handled within a single method call:
```rust
let pok = Prover::commit_signature_pok(&proof_request, proof_messages.as_slice(), &sig_obj)?;
```

### Step 2: The Challenge (Fiat-Shamir Heuristic)

Alice computes a cryptographic challenge $c$ by hashing the commitments, the public key, and a random `nonce`. 

* The $\parallel$ symbol means **concatenation**.

$$ c = H(\text{Commitments} \parallel PK \parallel \text{Nonce}) $$

**The Nonce is critical:** The Verifier provides a random `nonce` to ensure this specific proof cannot be copied and reused later by an attacker (Replay Attack). 

*Code Mapping:* First, the Verifier generates the nonce, then the Prover uses it to create the challenge:
```rust
// The Verifier generates a random, single-use nonce
let nonce = Verifier::generate_proof_nonce();

// The Prover concatenates everything and hashes it to create the challenge 'c'
let challenge = Prover::create_challenge_hash(&[pok.clone()], None, &nonce)?;
```

### Step 3: Generating Responses
Alice computes mathematical responses for the hidden messages ($m_1, m_3$) and the blinding factors using the challenge $c$. For a hidden message $m_i$, the response is calculated as $r_{m_i} = v_i - c \cdot m_i$ (where $v_i$ is a random blinding factor).

*Code Mapping:* 
```rust
// Packages the calculated responses into the final SignatureProof object
let sig = Prover::generate_signature_pok(pok, &challenge)?;
```

## 5. Verification (`main.rs`)

The Verifier receives the blinded signature components ($A', \bar{A}, D$), the challenge $c$, the responses, and the **revealed message** $m_2$ (`IsMajor`).

The Verifier cannot reconstruct the original Base Aggregate Point $P$ because they do not know Alice's hidden data or her signature blinding factor. Instead, the Verifier uses the challenge $c$ and the received cryptographic responses to mathematically rebuild Alice's commitments under the hood.

Finally, the Verifier uses the **bilinear pairing** function to check the signature's validity using the Issuer's Public Key $W$ ($W = x \cdot g_2$):

$$ e(A', W) \stackrel{?}{=} e(\bar{A}, g_2) $$

#### **Why does this equation match? (The Bilinear Proof)**

Let's resolve it to valdiate if it is working with the pairing properties and the definitions from Step 1:

1. **Expand the Left Side**:  

   We know $W = x \cdot g_2$ and $A' = r_1 \cdot A$. By using bilinearity, we can pull the scalar numbers ($r_1$ and $x$) outside the pairing:

$$ e(A', W) = e(r_1 \cdot A, x \cdot g_2) = e(A, g_2)^{r_1 \cdot x} $$

2. **Expand the Right Side**:

   We substitute $\bar{A}$ with its definition ($\bar{A} = r_1 \cdot P - e \cdot A'$) and simplify it using $A' = r_1 \cdot A$:

$$ e(\bar{A}, g_2) = e(r_1 \cdot P - e \cdot r_1 \cdot A, g_2) $$
$$ e(r_1 \cdot (P - e \cdot A), g_2) $$
$$ e(P - e \cdot A, g_2)^{r_1} $$

3. **The Final Link**:  
   The Issuer's fundamental signing equation is $(x + e) \cdot A = P$, which rearranges to $P - e \cdot A = x \cdot A$. If we substitute this into our right side:

$$ e(x \cdot A, g_2)^{r_1} = e(A, g_2)^{r_1 \cdot x} $$

We now have two equal values:

$$ e(A, g_2)^{r_1 \cdot x} = e(A, g_2)^{r_1 \cdot x} $$

*Code Mapping:* In `main.rs`, this verification and pairing math is executed with:
```rust
// Performs the cryptographic pairing check and extracts the validated disclosures
let verifier = Verifier::verify_signature_pok(
    &proof.proof_request, 
    &proof.signature, 
    &proof.nonce
).map_err(|e| e.to_string());
```

This prints the following output when executed, proving everything is mathematically and functionally sound:
```text
Identity: Identity { name: "Alice", is_major: true, address: "42-4567 crypto world", signature: "kbXeXSjmlxuU8iR28SEuSMhd6S0/3LD6RM1FbRBPB1OnENEF85VGrWTiS1YC5g+mUJyxeG4Dt51RhKBMH3vZbIng7Ygch0tVsZXe111YwG5gQMEbTLsL9PdPptrfqjVjSlqbwIHCrI9QtfrGm+U/xw==" }

Is Alice a major?
lets verify the proof and signature
Proof verified successfully, revealed message [SignatureMessage(Fr(0x1a6b1ab0601bbd319513c9b10e6204364b9b4f6238ccd4a360c4fd086518abd3))], expected message [SignatureMessage(Fr(0x1a6b1ab0601bbd319513c9b10e6204364b9b4f6238ccd4a360c4fd086518abd3))]
```
