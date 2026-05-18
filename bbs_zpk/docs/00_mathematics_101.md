# Mathematics 101: Elliptic Curves and Field Extensions

> [!NOTE]
> I am definitely not a professional mathematician, I just know the basics. I've tried to document what helped me get a grasp on the mathematics of the BBS+ signature scheme using bilinear pairings, and how it maps to the Rust implementation.
>
> These are my personal notes and current understanding, hopefully they will help someone else on the same journey!

Before diving into the rather complex mathematics of BBS+ signatures (see [Mathematics of BBS+](02_mathematics_of_bbs.md)), let's try to understand the basics of Elliptic Curves and Field Extensions with a simple example.

## Part 1: What is an elliptic curve? (The Basics)

An elliptic curve is a set of points $(x, y)$ that satisfy an equation of the form:

$$ y^{2} = x^{3} + Ax + B $$

In cryptography, the works is done on a prime finite field $\mathbb{F}_{p}$. This simply means that all coordinates $x$ and $y$ must be integers between $0$ and $p-1$, and we apply a modulo $p$ at each step of the calculation.

**Concrete Example: The curve $y^2 = x^3 + 1 \pmod 5$**

Here, our playground (the field $\mathbb{F}_{5}$) contains only 5 numbers: $\{0, 1, 2, 3, 4\}$.
Let's see if the point $(2, 3)$ belongs to this curve:
- Left side ($y^{2}$): $3^2 = 9$. And $9 \pmod 5 = \mathbf{4}$.
- Right side ($x^3 + 1$): $2^3 + 1 = 8 + 1 = 9$. And $9 \pmod 5 = \mathbf{4}$.

Since $4 = 4$, the point $(2, 3)$ is a valid point on our curve.

You can verify this using the following SageMath code from [`101.sage`](../sage/101.sage):
```python
p = 5
Fp = GF(p)
# Equation: y^2 = x^3 + 1
E = EllipticCurve(Fp, [0, 1])

print("The order of the curve is : ", E.order())
print("is (0,1) in the curve ? ", (0,1) in E) # True
print("is (1,4) in the curve ? ", (1,4) in E) # False
```

Let's verify why $(1, 4)$ is not on the curve:
- Left side: $4^2 = 16$. And $16 \pmod 5 = \mathbf{1}$.
- Right side: $1^3 + 1 = \mathbf{2}$.
Since $1 \neq 2$, the point $(1, 4)$ does not belong to the curve.

## Part 2: Why is an "Extension" needed?

Let's take our curve $y^2 = x^3 + 1 \pmod 5$ again. Let's try to see what happens if we choose the coordinate $x = 3$:
- Right side: $3^3 + 1 = 27 + 1 = 28 \equiv \mathbf{3} \pmod 5$.

So the equation becomes: $y^2 = 3 \pmod 5$.
Let's look for a number $y$ in our set $\{0, 1, 2, 3, 4\}$ which, multiplied by itself, gives $3$:
- $0^2 = 0 \neq 3$
- $1^2 = 1 \neq 3$
- $2^2 = 4 \neq 3$
- $3^2 = 9 \equiv 4 \pmod 5 \neq 3$
- $4^2 = 16 \equiv 1 \pmod 5 \neq 3$

**Observation**: There is no number in our set whose square is 3. For $x=3$, our curve has no point.

### The Solution: Create an imaginary number $i$

> [!NOTE]
> This part gets a bit trickier to understand, but it forms the core of the BBS+ signature scheme. It's similar to the approach that led to the creation of the imaginary number $i^{2}=-1$ in complex numbers $\mathbb{C}$ to find the square root of -1.
>
> There might be errors in my explanation below that I'm unaware of—this just reflects my current level of understanding!

Let's artificially "create" a magic number, let's call it $i$, and we decree that it respects a rule _(a defining polynomial)._

If we choose the **irreductible** polynomial $x^2 + 2$, we can check that it has no solution in $\mathbb{F}_5$ meaning $x \in \mathbb{F}_5$, $x^2 + 2 \neq 0$:

- $0^2 + 2 = 2 \neq 0$
- $1^2 + 2 = 3 \neq 0$
- $2^2 + 2 = 6 \equiv 1 \pmod 5 \neq 0$
- $3^2 + 2 = 11 \equiv 1 \pmod 5 \neq 0$
- $4^2 + 2 = 18 \equiv 3 \pmod 5 \neq 0$

> [!NOTE]
> The polynomial must be irreductible to create a field extension, that's a rule, otherwise can be factorized in smaller fields and it is not an extension.

We define the rule of this new field such that the root $i$ cancels the polynomial:

$$ i^2 + 2 = 0 \implies i^2 = -2 \implies i^2 \equiv 3 \pmod 5 $$

Unicorn land : in this new field, $i^2$ is exactly 3! Our equation $y^2 = 3$ finally has a solution: $y = i$.

By doing this, we have just created an **Extension of degree 2**, denoted $\mathbb{F}_{5^{2}}$.
From now on, our playground no longer contains only 5 numbers, but binomials of the form:

$$ a + bi $$

Where $a$ and $b$ are standard numbers between $0$ and $4$. Our space now contains $5 \times 5 = 25$ elements.

Let's look at the SageMath code to extend our curve:
```python
# Let's extend our curve to find a solution
R.<x> = PolynomialRing(Fp)
Fp2.<i> = Fp.extension(x^2 + 2)
E_Fp2 = EllipticCurve(Fp2, [0, 1])

print("is (3,i) in the curve E_Fp2 ? ", (3,i) in E_Fp2) # True
```

### Calculating in the Extension

Let's try calculating $(2 + 1i) \times (1 + 2i)$ in our new extension defined by $i^2 = 3 \pmod 5$:

$$ (2 \times 1) + (2 \times 2i) + (1i \times 1) + (1i \times 2i) $$
$$ = 2 + 5i + 2i^2 $$
$$ = 2 + 5i + 6 = 8 + 5i$$

2. **The field rule (Modulo 5)**: We reduce the real and imaginary coefficients modulo 5.
   - Real part: $8 \pmod 5 = \mathbf{3}$
   - Imaginary part: $5 \pmod 5 = \mathbf{0}$ (the $i$ term completely disappears!)

The final result is simply $3$ (or $3 + 0i$).

```python
# compute that in our extension Fp2 where i^2 = 3 mod 5
z1=Fp2(2+i)
z2=Fp2(1+2*i)

print("z1*z2 = ",z1*z2)
```

## Part 3: Bilinear Pairings and Shared Secrets

With our base curve $\mathbb{F}_p$ and its extension $\mathbb{F}_{p^2}$ set up, Lets play with **Bilinear Pairings**. A pairing (like the Tate pairing used in our script) is a mathematical operation that takes two points from our elliptic curves (one originating from the base curve and one from the extension) and maps them to a single value in a multiplicative target field.

> [!NOTE]
> Essentially, it maps a geometric operation _(like the addition of points)_ on the curve to polynomial arithmetic in a different field.

The property of a pairing is **bilinearity**, which means you can move the scalar multipliers out as exponents:

$$ e(a \cdot P, b \cdot Q) = e(P, Q)^{ab} $$

This allows two parties to establish a shared secret. In the [`101.sage`](../sage/101.sage) example, we demonstrate this:
1. **Setup**: We determine the max prime factor of the curve's order ($n$) and the degree of embedding ($k$).
2. **Points**: We pick a point $P$ on the base curve and $Q$ on the extension curve.
3. **Keys**: Alice and Bob choose private keys ($a=2$, $b=4$) and compute their public keys ($A = a \cdot P_{ext}$ and $B = b \cdot Q$).

Using the Tate pairing, they arrive at a common shared secret:

```python
# Setup parameters
facteurs = E.order().factor()
n = max([facteur for facteur, puissance in facteurs])
k = GF(n)(p).multiplicative_order()

# Points on base and extension curves
P = E((2, 2))
P_ext = E_Fp2(P)
Q = E_Fp2((1 + 2*i, i))

# Private and Public keys
a = 2
b = 4
A = a * P_ext
B = b * Q

# Shared secret generation using Tate pairing
Alice_shared_secret = A.tate_pairing(B, n, k)**a
Bob_shared_secret   = B.tate_pairing(A, n, k)**b
```

Running the script outputs the following, showing that both parties independently compute the exact same value (`4*i + 2`), thus successfully establishing a shared secret:

```text
max prime factor of the order of the curve :  3
degree of embedding :  2
Order of the subgroup (n) : 3
Degree of embedding (k)    : 2
Alice's shared secret e(pub_alice,  pub_bob) ^ priv_alice : 4*i + 2
Bob's shared secret e(pub_bob, pub_alice) ^ priv_bob   : 4*i + 2
```

This bilinearity property is the building block that allows the BBS+ signature scheme to securely verify zero-knowledge proofs.

## Part 4: Applying this to BLS12-381

By understanding the extension above, you have discovered the secret to constructing the BLS12-381 curve.
When you defined the rule $i^2 = 3$, you performed the exact same operation as this SageMath line for BLS12-381 in [`bilinearity.sage`](../sage/bilinearity.sage):
```python
Fp2.<i> = Fp.extension(x^2 + 1)
```
which imposes the rule $i^2 = -1$.

For BLS12-381, since the extension $\mathbb{F}_{p^{2}}$ is not enough to find all the points of the group $G_{2}$, mathematicians repeat this operation a second time by creating a new imaginary element $v$ on top of the first one, such that $v^6 = i + 1$. This is called a **Tower Extension** ($\mathbb{F}_{p^{12}}$). That is extracted from the [documentation](https://datatracker.ietf.org/doc/draft-irtf-cfrg-pairing-friendly-curves/08/) and where my understanding fails, let's take it as granted for now.

Here is the output from running our [`bilinearity.sage`](../sage/bilinearity.sage) script that proves these properties on the actual BLS12-381 curve:

```text
p bits : 381
r bits : 255
ïs pairing friendly (p**12 - 1) % r True
is E_Fp order prime ? False
is r prime ? True
is h1 cofactor = (z - 1)**2 // 3 True
is p_final in G1? True
r * p_final == E_Fp(0) True
G2 is well in the subgroup of order r ? True
G1 is well in the subgroup of order r ? True
is g1_12 in E_Fp12 ? True
is g2_12 in E_Fp12 ? True
pairing_left == pairing_right ? True
```

It could be sumup to take a point in the curve and project in the extension that is now a polynome in the extension that can be multiplied. it is what the bilinearity coupling do. 

$e(a.g1, b.g2) = e(g1, g2)^{ab}$