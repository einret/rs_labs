# Definition of BLS12_381 curve
# curve parameters definition https://www.johndcook.com/blog/2025/10/13/ethereum-bls12-381/
# another reference for curve parameter definition https://hackmd.io/@benjaminion/bls12-381
# https://eth2book.info/capella/part2/building_blocks/bls12-381/

z = -0xd201000000010000

# prime modulus of Fp where subgroup G1 and G2 live (381 bits)
# p = (1/3)*(z-1)**2*(z**4-z**2+1)+z


p = ZZ((z-1)**2*(z**4 - z**2 + 1)//3 + z)
print("p bits :",p.nbits())

# compute subgroup (255 bits) r= order of the subgroup which is prime.
r = ZZ(z**4 - z**2 + 1)
print("r bits :",r.nbits())

print("ïs pairing friendly (p**12 - 1) % r", (p**12 - 1) % r == 0)

# Equation of the curve BLS12-381 is y^2 = x^3 + 4
Fp = GF(p)
E_Fp = EllipticCurve(Fp, [0, 4])

print("is E_Fp order prime ?", E_Fp.order().is_prime())
print("is r prime ?", r.is_prime())

# compute h1 which is the cofactor meaning that h1=N/r with N the number of point in E(Fp) and r the order of the subgroup G1
# https://eprint.iacr.org/2019/814.pdf
h1_reel = ZZ( E_Fp.order() / r)
h1_formule = ZZ((z - 1)**2 // 3)

print("is h1 cofactor = (z - 1)**2 // 3", h1_reel==h1_formule)

p_random= E_Fp.random_point()
p_final = p_random * h1
print("is p_final in G1?", p_final in E_Fp)
print ("r * p_final == E_Fp(0)" ,r * p_final == E_Fp(0))

