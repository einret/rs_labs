# Definition of BLS12_381 curve
# curve parameters definition https://www.johndcook.com/blog/2025/10/13/ethereum-bls12-381/
# another reference for curve parameter definition https://hackmd.io/@benjaminion/bls12-381
# https://eth2book.info/capella/part2/building_blocks/bls12-381/

# seed
z = -0xd201000000010000

# constante and polynomial of the curve
p = ZZ((z-1)**2*(z**4 - z**2 + 1)//3 + z)
r = ZZ(z**4 - z**2 + 1)

Fp = GF(p)

# definition of the curve on Fp
E_Fp = EllipticCurve(Fp, [0, 4])
# co-factor h1 used to clean points of E_Fp to be on subgroup of order r
h1 = ZZ((z - 1)**2 // 3)


# definition of the curve on Extention Fp**2 
# extension is defined by i^2+1=0 -> x^2+1=0
R1.<x> = PolynomialRing(Fp)
Fp2.<i> = Fp.extension(x^2 + 1)

# Equation of the curve for G2 is y^2 = x^3 + 4(i + 1)
E_Fp2 = EllipticCurve(Fp2, [0, 4*(i + 1)])
# co-factor h2 used to clean points of E_Fp2 to be on subgroup of order r
h2 = ZZ((z^8 - 4*z^7 + 5*z^6 - 4*z^4 + 6*z^3 - 4*z^2 - 4*z + 13) // 9)

# lets create the 2 generators G1 and G2
g1 = E_Fp.random_point()  * h1
g2 = E_Fp2.random_point() * h2

# quick check G2 is well in the subgroup of order r
print("G2 is well in the subgroup of order r ?", r * g2 == E_Fp2(0))
print("G1 is well in the subgroup of order r ?", r * g1 == E_Fp(0))


# limit reach for my maths background, out of my league, just apply https://eprint.iacr.org/2019/814.pdf

# The Unified Field Fp12 (Tower Construction)
# We define polynomial ring on Fp2
R.<w> = PolynomialRing(Fp2)

# We declare Fp12 as an extension of Fp2 (degree 2 * degree 6 = degree 12)
Fp12.<v> = Fp2.extension(w^6 - (i + 1))
E_Fp12 = EllipticCurve(Fp12, [0, 4])

# Mapping G1 (Fp -> Fp12) : trivial because Fp is naturally injected
g1_12 = E_Fp12(Fp12(g1[0]), Fp12(g1[1]))

# Mapping G2 (Fp2 -> Fp12) : Strict application of the Sextic Twist
# According to the theory of Type D Twist for BLS12-381 :
# x_g12 = x_g2 / v^2   and   y_g12 = y_g2 / v^3
# (v being the 6th root of i+1)
x_g2_12 = Fp12(g2[0]) / v^2
y_g2_12 = Fp12(g2[1]) / v^3
g2_12 = E_Fp12(x_g2_12, y_g2_12)

print("is g1_12 in E_Fp12 ?", g1_12 in E_Fp12)
print("is g2_12 in E_Fp12 ?", g2_12 in E_Fp12)


# private key
a = randint(1, r-1)
b = randint(1, r-1)

P_a= a* g1_12
Q_b= b* g2_12

# lets check that e(aG1, bG2) = e(G1, G2)**(a*b)
pairing_left = P_a.tate_pairing(Q_b, r, 12, q=p)
pairing_right_base = g1_12.tate_pairing(g2_12, r, 12, q=p)
pairing_right = pairing_right_base**(a*b)


print("pairing_left == pairing_right ?", pairing_left == pairing_right)
