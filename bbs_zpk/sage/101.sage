p=5
Fp=GF(p)
# y^2 = x^3 + 1
E=EllipticCurve(Fp,[0,1])

print("The order of the curve is : ",E.order())
print("is (0,1) in the curve ? ", (0,1) in E)
print("is (1,4) in the curve ? ", (1,4) in E)

# (1,4) do not belong to the curve E_Fp
# left side : 4^2 = 16. And 16 mod 5 = 1
# right side : 1^3 + 1 = 2
# 1 != 2 => The point (1, 4) does not belong to the curve.

# if x=3 
# 3^3 + 1 = 28 mod 5 = 3
# we need to find y^2=3 mod 5
# if we choose the polynomial x^2 + 2. Let's check if it is irreducible (no solution in F5):
# 0^2 + 2 = 2 != 0
# ...
# 4^2 + 2 = 16 + 2 = 18 = 3 (mod 5) != 0
# The rule of this new field is that the root i cancels the polynomial:
# i^2 + 2 = 0 => i^2 = -2 => i^2 = 3 (mod 5)
# Look at the miracle: in this new field, i^2 is exactly 3!

# lets extend or curve to find a solution
R.<x>= PolynomialRing(Fp)
Fp2.<i> = Fp.extension(x^2 + 2)
E_Fp2 = EllipticCurve(Fp2, [0, 1])

print("is (3,i) in the curve E_Fp2 ? ", (3,i) in E_Fp2)

z1=Fp2(2+i)
z2=Fp2(1+2*i)

print("z1*z2 = ",z1*z2)


# demo to generate a common secret using pairings
print()

# n = max prime factor of the order of the curve
facteurs = E.order().factor()
n = max([facteur for facteur, puissance in facteurs])
print("max prime factor of the order of the curve : ", n)

# k = degree of embedding (the smallest k such that p^k = 1 mod n)
k = GF(n)(p).multiplicative_order()
print("degree of embedding : ", k)
print(f"Order of the subgroup (n) : {n}")
print(f"Degree of embedding (k)    : {k}")

cofacteur_base = E.order() // n
cofacteur_ext  = E_Fp2.order() // n

# P is stritly on the base curve E 
P = E((2, 2))
P_ext = E_Fp2(P)

# Q is strictly in the extension E_Fp2 (with a component in 'i')
Q = E_Fp2((1 + 2*i, i))

# private keys
a= 2
b= 4

# public keys
A = a*P_ext
B = b*Q

# shared secret
# https://doc.sagemath.org/html/en/reference/arithmetic_curves/sage/schemes/elliptic_curves/ell_point.html#sage.schemes.elliptic_curves.ell_point.EllipticCurvePoint_field.tate_pairing
Alice_shared_secret = A.tate_pairing(B,n,k)**a
Bob_shared_secret   = B.tate_pairing(A,n,k)**b

print(f"Alice's shared secret e(pub_alice,  pub_bob) ^ priv_alice : {Alice_shared_secret}")
print(f"Bob's shared secret e(pub_bob, pub_alice) ^ priv_bob   : {Bob_shared_secret}")
