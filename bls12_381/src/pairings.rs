use crate::fp::Fp;
use crate::fp12::Fp12;
use crate::fp2::Fp2;
use crate::fp6::Fp6;
use crate::{G1Affine, G1Projective, G2Affine, G2Projective, Scalar, BLS_X, BLS_X_IS_NEGATIVE};

use core::borrow::Borrow;
use core::fmt;
use core::iter::Sum;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use group::Group;
use pairing::{Engine, PairingCurveAffine};
use rand_core::RngCore;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use pairing::MultiMillerLoop;

/// Represents results of a Miller loop, one of the most expensive portions
/// of the pairing function. `MillerLoopResult`s cannot be compared with each
/// other until `.final_exponentiation()` is called, which is also expensive.
#[cfg_attr(docsrs, doc(cfg(feature = "pairings")))]
#[derive(Copy, Clone, Debug)]
pub struct MillerLoopResult(pub(crate) Fp12);

impl Default for MillerLoopResult {
    fn default() -> Self {
        MillerLoopResult(Fp12::one())
    }
}

#[cfg(feature = "zeroize")]
impl zeroize::DefaultIsZeroes for MillerLoopResult {}

impl ConditionallySelectable for MillerLoopResult {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        MillerLoopResult(Fp12::conditional_select(&a.0, &b.0, choice))
    }
}

impl MillerLoopResult {
    /// This performs a "final exponentiation" routine to convert the result
    /// of a Miller loop into an element of `Gt` with help of efficient squaring
    /// operation in the so-called `cyclotomic subgroup` of `Fq6` so that
    /// it can be compared with other elements of `Gt`.
    pub fn final_exponentiation(&self) -> Gt {
        #[must_use]
        fn fp4_square(a: Fp2, b: Fp2) -> (Fp2, Fp2) {
            cfg_if::cfg_if! {
                if #[cfg(target_vendor = "succinct")] {
                    // c0 = b.square().mul_by_nonresidue() + a.square()
                    // c1 = (a + b).square() - a.square() - b.square()
                    let mut t0 = a;
                    t0.square_inp();
                    let mut c0 = b;
                    c0.square_inp();
                    let mut c1 = a;
                    c1.add_inp(&b);
                    c1.square_inp();
                    c1.sub_inp(&t0);
                    c1.sub_inp(&c0);
                    c0.mul_by_nonresidue_inp();
                    c0.add_inp(&t0);
                    (c0, c1)
                } else {
                    let t0 = a.square();
                    let t1 = b.square();
                    let mut t2 = t1.mul_by_nonresidue();
                    let c0 = t2 + t0;
                    t2 = a + b;
                    t2 = t2.square();
                    t2 -= t0;
                    let c1 = t2 - t1;
                    (c0, c1)
                }
            }
        }
        // Adaptation of Algorithm 5.5.4, Guide to Pairing-Based Cryptography
        // Faster Squaring in the Cyclotomic Subgroup of Sixth Degree Extensions
        // https://eprint.iacr.org/2009/565.pdf
        #[must_use]
        fn cyclotomic_square(f: Fp12) -> Fp12 {
            let Fp12 {
                c0:
                    Fp6 {
                        c0: mut z0,
                        c1: mut z4,
                        c2: mut z3,
                    },
                c1:
                    Fp6 {
                        c0: mut z2,
                        c1: mut z1,
                        c2: mut z5,
                    },
            } = f;

            let (t0, t1) = fp4_square(z0, z1);

            // For A
            cfg_if::cfg_if! {
                if #[cfg(target_vendor = "succinct")] {
                    z0 = -z0;
                    z0.add_inp(&t0);
                    z0.double_inp();
                    z0.add_inp(&t0);

                    z1.add_inp(&t1);
                    z1.double_inp();
                    z1.add_inp(&t1);
                } else {
                    z0 = t0 - z0;
                    z0 = z0 + z0 + t0;

                    z1 = t1 + z1;
                    z1 = z1 + z1 + t1;
                }
            }

            cfg_if::cfg_if! {
                if #[cfg(target_vendor = "succinct")] {
                    let (t0, t1) = fp4_square(z2, z3);
                    let (t2, mut t3) = fp4_square(z4, z5);
                } else {
                    let (mut t0, t1) = fp4_square(z2, z3);
                    let (t2, t3) = fp4_square(z4, z5);
                }
            }

            // For C
            cfg_if::cfg_if! {
                if #[cfg(target_vendor = "succinct")] {
                    z4 = -z4;
                    z4.add_inp(&t0);
                    z4.double_inp();
                    z4.add_inp(&t0);

                    z5.add_inp(&t1);
                    z5.double_inp();
                    z5.add_inp(&t1);
                } else {
                    z4 = t0 - z4;
                    z4 = z4 + z4 + t0;

                    z5 = t1 + z5;
                    z5 = z5 + z5 + t1;
                }
            }

            // For B
            cfg_if::cfg_if! {
                if #[cfg(target_vendor = "succinct")] {
                    t3.mul_by_nonresidue_inp();
                    z2.add_inp(&t3);
                    z2.double_inp();
                    z2.add_inp(&t3);

                    z3 = -z3;
                    z3.add_inp(&t2);
                    z3.double_inp();
                    z3.add_inp(&t2);
                } else {
                    t0 = t3.mul_by_nonresidue();
                    z2 = t0 + z2;
                    z2 = z2 + z2 + t0;

                    z3 = t2 - z3;
                    z3 = z3 + z3 + t2;
                }
            }

            Fp12 {
                c0: Fp6 {
                    c0: z0,
                    c1: z4,
                    c2: z3,
                },
                c1: Fp6 {
                    c0: z2,
                    c1: z1,
                    c2: z5,
                },
            }
        }
        #[cfg(target_vendor = "succinct")]
        fn cyclotomic_square_inp(f: &mut Fp12) {
            // z0: f.c0.c0
            // z1: f.c1.c1
            // z2: f.c1.c0
            // z3: f.c0.c2
            // z4: f.c0.c1
            // z5: f.c1.c2

            let (t0, t1) = fp4_square(f.c0.c0, f.c1.c1);

            // For A
            f.c0.c0 = -f.c0.c0;
            f.c0.c0.add_inp(&t0);
            f.c0.c0.double_inp();
            f.c0.c0.add_inp(&t0);

            f.c1.c1.add_inp(&t1);
            f.c1.c1.double_inp();
            f.c1.c1.add_inp(&t1);

            let (t0, t1) = fp4_square(f.c1.c0, f.c0.c2);
            let (t2, mut t3) = fp4_square(f.c0.c1, f.c1.c2);

            // For C
            f.c0.c1 = -f.c0.c1;
            f.c0.c1.add_inp(&t0);
            f.c0.c1.double_inp();
            f.c0.c1.add_inp(&t0);

            f.c1.c2.add_inp(&t1);
            f.c1.c2.double_inp();
            f.c1.c2.add_inp(&t1);

            // For B
            t3.mul_by_nonresidue_inp();
            f.c1.c0.add_inp(&t3);
            f.c1.c0.double_inp();
            f.c1.c0.add_inp(&t3);

            f.c0.c2 = -f.c0.c2;
            f.c0.c2.add_inp(&t2);
            f.c0.c2.double_inp();
            f.c0.c2.add_inp(&t2);
        }
        #[must_use]
        fn cycolotomic_exp(f: &Fp12) -> Fp12 {
            let x = BLS_X;
            let mut tmp = Fp12::one();
            let mut found_one = false;
            for i in (0..64).rev().map(|b| ((x >> b) & 1) == 1) {
                if found_one {
                    cfg_if::cfg_if! {
                        if #[cfg(target_vendor = "succinct")] {
                            cyclotomic_square_inp(&mut tmp);
                        } else {
                            tmp = cyclotomic_square(tmp);
                        }
                    }
                } else {
                    found_one = i;
                }

                if i {
                    cfg_if::cfg_if! {
                        if #[cfg(target_vendor = "succinct")] {
                            tmp.mul_inp(f);
                        } else {
                            tmp *= f;
                        }
                    }
                }
            }

            tmp.conjugate()
        }

        cfg_if::cfg_if! {
            if #[cfg(target_vendor = "succinct")] {
                let mut t0 = self.0;
                t0.frobenius_map_inp();
                t0.frobenius_map_inp();
                t0.frobenius_map_inp();
                t0.frobenius_map_inp();
                t0.frobenius_map_inp();
                t0.frobenius_map_inp();

                let mut t2 = self.0.invert().unwrap();
                t2.mul_inp(&t0);
                let mut t1 = t2;
                t2.frobenius_map_inp();
                t2.frobenius_map_inp();
                t2.mul_inp(&t1);
                t1 = cyclotomic_square(t2);
                t1.conjugate_inp();
                let mut t3 = cycolotomic_exp(&t2);
                let mut t4 = cyclotomic_square(t3);
                let mut t5 = t1;
                t5.mul_inp(&t3);
                t1 = cycolotomic_exp(&t5);
                t0 = cycolotomic_exp(&t1);
                let mut t6 = cycolotomic_exp(&t0);
                t6.mul_inp(&t4);
                t4 = cycolotomic_exp(&t6);
                t5.conjugate_inp();
                t4.mul_inp(&t5);
                t4.mul_inp(&t2);
                t1.mul_inp(&t2);
                t2.conjugate_inp();
                t1.frobenius_map_inp();
                t1.frobenius_map_inp();
                t1.frobenius_map_inp();
                t6.mul_inp(&t2);
                t6.frobenius_map_inp();
                t3.mul_inp(&t0);
                t3.frobenius_map_inp();
                t3.frobenius_map_inp();
                t3.mul_inp(&t1);
                t3.mul_inp(&t6);
                t3.mul_inp(&t4);
                Gt(t3)
            } else {
                let mut t0 = self
                    .0
                    .frobenius_map()
                    .frobenius_map()
                    .frobenius_map()
                    .frobenius_map()
                    .frobenius_map()
                    .frobenius_map();

                // We unwrap() because `MillerLoopResult` can only be constructed
                // by a function within this crate, and we uphold the invariant
                // that the enclosed value is nonzero.
                let mut t1 = self.0.invert().unwrap();

                let mut t2 = t0 * t1;
                t1 = t2;
                t2 = t2.frobenius_map().frobenius_map();
                t2 *= t1;
                t1 = cyclotomic_square(t2).conjugate();
                let mut t3 = cycolotomic_exp(&t2);
                let mut t4 = cyclotomic_square(t3);
                let mut t5 = t1 * t3;
                t1 = cycolotomic_exp(&t5);
                t0 = cycolotomic_exp(&t1);
                let mut t6 = cycolotomic_exp(&t0);
                t6 *= t4;
                t4 = cycolotomic_exp(&t6);
                t5 = t5.conjugate();
                t4 *= t5 * t2;
                t5 = t2.conjugate();
                t1 *= t2;
                t1 = t1.frobenius_map().frobenius_map().frobenius_map();
                t6 *= t5;
                t6 = t6.frobenius_map();
                t3 *= t0;
                t3 = t3.frobenius_map().frobenius_map();
                t3 *= t1;
                t3 *= t6;
                Gt(t3 * t4)
            }
        }
    }
}

impl<'a, 'b> Add<&'b MillerLoopResult> for &'a MillerLoopResult {
    type Output = MillerLoopResult;

    #[inline]
    fn add(self, rhs: &'b MillerLoopResult) -> MillerLoopResult {
        MillerLoopResult(self.0 * rhs.0)
    }
}

impl_add_binop_specify_output!(MillerLoopResult, MillerLoopResult, MillerLoopResult);

impl AddAssign<MillerLoopResult> for MillerLoopResult {
    #[inline]
    fn add_assign(&mut self, rhs: MillerLoopResult) {
        *self = *self + rhs;
    }
}

impl<'b> AddAssign<&'b MillerLoopResult> for MillerLoopResult {
    #[inline]
    fn add_assign(&mut self, rhs: &'b MillerLoopResult) {
        *self = *self + rhs;
    }
}

/// This is an element of $\mathbb{G}_T$, the target group of the pairing function. As with
/// $\mathbb{G}_1$ and $\mathbb{G}_2$ this group has order $q$.
///
/// Typically, $\mathbb{G}_T$ is written multiplicatively but we will write it additively to
/// keep code and abstractions consistent.
#[cfg_attr(docsrs, doc(cfg(feature = "pairings")))]
#[derive(Copy, Clone, Debug)]
pub struct Gt(pub Fp12);

impl Default for Gt {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(feature = "zeroize")]
impl zeroize::DefaultIsZeroes for Gt {}

impl fmt::Display for Gt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ConstantTimeEq for Gt {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl ConditionallySelectable for Gt {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Gt(Fp12::conditional_select(&a.0, &b.0, choice))
    }
}

impl Eq for Gt {}
impl PartialEq for Gt {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        bool::from(self.ct_eq(other))
    }
}

impl Gt {
    /// Returns the group identity, which is $1$.
    pub fn identity() -> Gt {
        Gt(Fp12::one())
    }

    /// Doubles this group element.
    pub fn double(&self) -> Gt {
        Gt(self.0.square())
    }
}

impl<'a> Neg for &'a Gt {
    type Output = Gt;

    #[inline]
    fn neg(self) -> Gt {
        // The element is unitary, so we just conjugate.
        Gt(self.0.conjugate())
    }
}

impl Neg for Gt {
    type Output = Gt;

    #[inline]
    fn neg(self) -> Gt {
        -&self
    }
}

impl<'a, 'b> Add<&'b Gt> for &'a Gt {
    type Output = Gt;

    #[inline]
    fn add(self, rhs: &'b Gt) -> Gt {
        Gt(self.0 * rhs.0)
    }
}

impl<'a, 'b> Sub<&'b Gt> for &'a Gt {
    type Output = Gt;

    #[inline]
    fn sub(self, rhs: &'b Gt) -> Gt {
        self + (-rhs)
    }
}

impl<'a, 'b> Mul<&'b Scalar> for &'a Gt {
    type Output = Gt;

    fn mul(self, other: &'b Scalar) -> Self::Output {
        let mut acc = Gt::identity();

        // This is a simple double-and-add implementation of group element
        // multiplication, moving from most significant to least
        // significant bit of the scalar.
        //
        // We skip the leading bit because it's always unset for Fq
        // elements.
        for bit in other
            .to_bytes()
            .iter()
            .rev()
            .flat_map(|byte| (0..8).rev().map(move |i| Choice::from((byte >> i) & 1u8)))
            .skip(1)
        {
            acc = acc.double();
            acc = Gt::conditional_select(&acc, &(acc + self), bit);
        }

        acc
    }
}

impl_binops_additive!(Gt, Gt);
impl_binops_multiplicative!(Gt, Scalar);

impl<T> Sum<T> for Gt
where
    T: Borrow<Gt>,
{
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        iter.fold(Self::identity(), |acc, item| acc + item.borrow())
    }
}

impl Group for Gt {
    type Scalar = Scalar;

    fn random(mut rng: impl RngCore) -> Self {
        loop {
            let inner = Fp12::random(&mut rng);

            // Not all elements of Fp12 are elements of the prime-order multiplicative
            // subgroup. We run the random element through final_exponentiation to obtain
            // a valid element, which requires that it is non-zero.
            if !bool::from(inner.is_zero()) {
                return MillerLoopResult(inner).final_exponentiation();
            }
        }
    }

    fn identity() -> Self {
        Self::identity()
    }

    fn generator() -> Self {
        // pairing(&G1Affine::generator(), &G2Affine::generator())
        Gt(Fp12 {
            c0: Fp6 {
                c0: Fp2 {
                    c0: Fp::from_raw_unchecked([
                        0x1972_e433_a01f_85c5,
                        0x97d3_2b76_fd77_2538,
                        0xc8ce_546f_c96b_cdf9,
                        0xcef6_3e73_66d4_0614,
                        0xa611_3427_8184_3780,
                        0x13f3_448a_3fc6_d825,
                    ]),
                    c1: Fp::from_raw_unchecked([
                        0xd263_31b0_2e9d_6995,
                        0x9d68_a482_f779_7e7d,
                        0x9c9b_2924_8d39_ea92,
                        0xf480_1ca2_e131_07aa,
                        0xa16c_0732_bdbc_b066,
                        0x083c_a4af_ba36_0478,
                    ]),
                },
                c1: Fp2 {
                    c0: Fp::from_raw_unchecked([
                        0x59e2_61db_0916_b641,
                        0x2716_b6f4_b23e_960d,
                        0xc8e5_5b10_a0bd_9c45,
                        0x0bdb_0bd9_9c4d_eda8,
                        0x8cf8_9ebf_57fd_aac5,
                        0x12d6_b792_9e77_7a5e,
                    ]),
                    c1: Fp::from_raw_unchecked([
                        0x5fc8_5188_b0e1_5f35,
                        0x34a0_6e3a_8f09_6365,
                        0xdb31_26a6_e02a_d62c,
                        0xfc6f_5aa9_7d9a_990b,
                        0xa12f_55f5_eb89_c210,
                        0x1723_703a_926f_8889,
                    ]),
                },
                c2: Fp2 {
                    c0: Fp::from_raw_unchecked([
                        0x9358_8f29_7182_8778,
                        0x43f6_5b86_11ab_7585,
                        0x3183_aaf5_ec27_9fdf,
                        0xfa73_d7e1_8ac9_9df6,
                        0x64e1_76a6_a64c_99b0,
                        0x179f_a78c_5838_8f1f,
                    ]),
                    c1: Fp::from_raw_unchecked([
                        0x672a_0a11_ca2a_ef12,
                        0x0d11_b9b5_2aa3_f16b,
                        0xa444_12d0_699d_056e,
                        0xc01d_0177_221a_5ba5,
                        0x66e0_cede_6c73_5529,
                        0x05f5_a71e_9fdd_c339,
                    ]),
                },
            },
            c1: Fp6 {
                c0: Fp2 {
                    c0: Fp::from_raw_unchecked([
                        0xd30a_88a1_b062_c679,
                        0x5ac5_6a5d_35fc_8304,
                        0xd0c8_34a6_a81f_290d,
                        0xcd54_30c2_da37_07c7,
                        0xf0c2_7ff7_8050_0af0,
                        0x0924_5da6_e2d7_2eae,
                    ]),
                    c1: Fp::from_raw_unchecked([
                        0x9f2e_0676_791b_5156,
                        0xe2d1_c823_4918_fe13,
                        0x4c9e_459f_3c56_1bf4,
                        0xa3e8_5e53_b9d3_e3c1,
                        0x820a_121e_21a7_0020,
                        0x15af_6183_41c5_9acc,
                    ]),
                },
                c1: Fp2 {
                    c0: Fp::from_raw_unchecked([
                        0x7c95_658c_2499_3ab1,
                        0x73eb_3872_1ca8_86b9,
                        0x5256_d749_4774_34bc,
                        0x8ba4_1902_ea50_4a8b,
                        0x04a3_d3f8_0c86_ce6d,
                        0x18a6_4a87_fb68_6eaa,
                    ]),
                    c1: Fp::from_raw_unchecked([
                        0xbb83_e71b_b920_cf26,
                        0x2a52_77ac_92a7_3945,
                        0xfc0e_e59f_94f0_46a0,
                        0x7158_cdf3_7860_58f7,
                        0x7cc1_061b_82f9_45f6,
                        0x03f8_47aa_9fdb_e567,
                    ]),
                },
                c2: Fp2 {
                    c0: Fp::from_raw_unchecked([
                        0x8078_dba5_6134_e657,
                        0x1cd7_ec9a_4399_8a6e,
                        0xb1aa_599a_1a99_3766,
                        0xc9a0_f62f_0842_ee44,
                        0x8e15_9be3_b605_dffa,
                        0x0c86_ba0d_4af1_3fc2,
                    ]),
                    c1: Fp::from_raw_unchecked([
                        0xe80f_f2a0_6a52_ffb1,
                        0x7694_ca48_721a_906c,
                        0x7583_183e_03b0_8514,
                        0xf567_afdd_40ce_e4e2,
                        0x9a6d_96d2_e526_a5fc,
                        0x197e_9f49_861f_2242,
                    ]),
                },
            },
        })
    }

    fn is_identity(&self) -> Choice {
        self.ct_eq(&Self::identity())
    }

    #[must_use]
    fn double(&self) -> Self {
        self.double()
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "pairings", feature = "alloc"))))]
#[derive(Clone, Debug)]
/// This structure contains cached computations pertaining to a $\mathbb{G}_2$
/// element as part of the pairing function (specifically, the Miller loop) and
/// so should be computed whenever a $\mathbb{G}_2$ element is being used in
/// multiple pairings or is otherwise known in advance. This should be used in
/// conjunction with the [`multi_miller_loop`](crate::multi_miller_loop)
/// function provided by this crate.
///
/// Requires the `alloc` and `pairing` crate features to be enabled.
pub struct G2Prepared {
    infinity: Choice,
    coeffs: Vec<(Fp2, Fp2, Fp2)>,
}

#[cfg(feature = "alloc")]
impl From<G2Affine> for G2Prepared {
    fn from(q: G2Affine) -> G2Prepared {
        struct Adder {
            cur: G2Projective,
            base: G2Affine,
            coeffs: Vec<(Fp2, Fp2, Fp2)>,
        }

        impl MillerLoopDriver for Adder {
            type Output = ();

            fn doubling_step(&mut self, _: &mut Self::Output) {
                let coeffs = doubling_step(&mut self.cur);
                self.coeffs.push(coeffs);
            }
            fn addition_step(&mut self, _: &mut Self::Output) {
                let coeffs = addition_step(&mut self.cur, &self.base);
                self.coeffs.push(coeffs);
            }
            fn square_output(_: &mut Self::Output) {}
            fn conjugate(_: &mut Self::Output) {}
            fn one() -> Self::Output {}
        }

        let is_identity = q.is_identity();
        let q = G2Affine::conditional_select(&q, &G2Affine::generator(), is_identity);

        let mut adder = Adder {
            cur: G2Projective::from(q),
            base: q,
            coeffs: Vec::with_capacity(68),
        };

        miller_loop(&mut adder);

        assert_eq!(adder.coeffs.len(), 68);

        G2Prepared {
            infinity: is_identity,
            coeffs: adder.coeffs,
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "pairings", feature = "alloc"))))]
/// Computes $$\sum_{i=1}^n \textbf{ML}(a_i, b_i)$$ given a series of terms
/// $$(a_1, b_1), (a_2, b_2), ..., (a_n, b_n).$$
///
/// Requires the `alloc` and `pairing` crate features to be enabled.
pub fn multi_miller_loop(terms: &[(&G1Affine, &G2Prepared)]) -> MillerLoopResult {
    struct Adder<'a, 'b, 'c> {
        terms: &'c [(&'a G1Affine, &'b G2Prepared)],
        index: usize,
    }

    impl<'a, 'b, 'c> MillerLoopDriver for Adder<'a, 'b, 'c> {
        type Output = Fp12;

        fn doubling_step(&mut self, f: &mut Self::Output) {
            let index = self.index;
            for term in self.terms {
                let either_identity = term.0.is_identity() | term.1.infinity;

                let new_f = ell(f, &term.1.coeffs[index], term.0);
                *f = Fp12::conditional_select(&new_f, f, either_identity);
            }
            self.index += 1;
        }
        fn addition_step(&mut self, f: &mut Self::Output) {
            let index = self.index;
            for term in self.terms {
                let either_identity = term.0.is_identity() | term.1.infinity;

                let new_f = ell(f, &term.1.coeffs[index], term.0);
                *f = Fp12::conditional_select(&new_f, f, either_identity);
            }
            self.index += 1;
        }
        fn square_output(f: &mut Self::Output) {
            *f = f.square();
        }
        fn conjugate(f: &mut Self::Output) {
            f.conjugate_inp();
        }
        fn one() -> Self::Output {
            Fp12::one()
        }
    }

    let mut adder = Adder { terms, index: 0 };

    MillerLoopResult(miller_loop(&mut adder))
}

/// Invoke the pairing function without the use of precomputation and other optimizations.
#[cfg_attr(docsrs, doc(cfg(feature = "pairings")))]
pub fn pairing(p: &G1Affine, q: &G2Affine) -> Gt {
    struct Adder {
        cur: G2Projective,
        base: G2Affine,
        p: G1Affine,
    }

    impl MillerLoopDriver for Adder {
        type Output = Fp12;

        fn doubling_step(&mut self, f: &mut Self::Output) {
            let coeffs = doubling_step(&mut self.cur);
            *f = ell(f, &coeffs, &self.p);
        }
        fn addition_step(&mut self, f: &mut Self::Output) {
            let coeffs = addition_step(&mut self.cur, &self.base);
            *f = ell(f, &coeffs, &self.p);
        }
        fn square_output(f: &mut Self::Output) {
            *f = f.square();
        }
        fn conjugate(f: &mut Self::Output) {
            f.conjugate_inp();
        }
        fn one() -> Self::Output {
            Fp12::one()
        }
    }

    let either_identity = p.is_identity() | q.is_identity();
    let p = G1Affine::conditional_select(p, &G1Affine::generator(), either_identity);
    let q = G2Affine::conditional_select(q, &G2Affine::generator(), either_identity);

    let mut adder = Adder {
        cur: G2Projective::from(q),
        base: q,
        p,
    };

    let tmp = miller_loop(&mut adder);
    let tmp = MillerLoopResult(Fp12::conditional_select(
        &tmp,
        &Fp12::one(),
        either_identity,
    ));
    tmp.final_exponentiation()
}

trait MillerLoopDriver {
    type Output;

    fn doubling_step(&mut self, f: &mut Self::Output);
    fn addition_step(&mut self, f: &mut Self::Output);
    fn square_output(f: &mut Self::Output);
    fn conjugate(f: &mut Self::Output);
    fn one() -> Self::Output;
}

/// This is a "generic" implementation of the Miller loop to avoid duplicating code
/// structure elsewhere; instead, we'll write concrete instantiations of
/// `MillerLoopDriver` for whatever purposes we need (such as caching modes).
fn miller_loop<D: MillerLoopDriver>(driver: &mut D) -> D::Output {
    let mut f = D::one();

    let mut found_one = false;
    for i in (0..64).rev().map(|b| (((BLS_X >> 1) >> b) & 1) == 1) {
        if !found_one {
            found_one = i;
            continue;
        }

        driver.doubling_step(&mut f);

        if i {
            driver.addition_step(&mut f);
        }

        D::square_output(&mut f);
    }

    driver.doubling_step(&mut f);

    if BLS_X_IS_NEGATIVE {
        D::conjugate(&mut f);
    }

    f
}

#[cfg(target_vendor = "succinct")]
fn ell(f: &Fp12, coeffs: &(Fp2, Fp2, Fp2), p: &G1Affine) -> Fp12 {
    let mut c0 = coeffs.0;
    let mut c1 = coeffs.1;

    c0.c0.mul_inp(&p.y);
    c0.c1.mul_inp(&p.y);

    c1.c0.mul_inp(&p.x);
    c1.c1.mul_inp(&p.x);

    f.mul_by_014(&coeffs.2, &c1, &c0)
}

#[cfg(not(target_vendor = "succinct"))]
fn ell(f: &Fp12, coeffs: &(Fp2, Fp2, Fp2), p: &G1Affine) -> Fp12 {
    let mut c0 = coeffs.0;
    let mut c1 = coeffs.1;

    c0.c0 *= p.y;
    c0.c1 *= p.y;

    c1.c0 *= p.x;
    c1.c1 *= p.x;

    f.mul_by_014(&coeffs.2, &c1, &c0)
}

#[cfg(target_vendor = "succinct")]
fn doubling_step(r: &mut G2Projective) -> (Fp2, Fp2, Fp2) {
    // Adaptation of Algorithm 26, https://eprint.iacr.org/2010/354.pdf
    let mut tmp0 = r.x;
    tmp0.square_inp();
    let mut tmp1 = r.y;
    tmp1.square_inp();
    let mut tmp2 = tmp1;
    tmp2.square_inp();
    let mut tmp3 = tmp1;
    tmp3.add_inp(&r.x);
    tmp3.square_inp();
    tmp3.sub_inp(&tmp0);
    tmp3.sub_inp(&tmp2);
    tmp3.double_inp();
    let mut tmp4 = tmp0;
    tmp4.add_inp(&tmp0);
    tmp4.add_inp(&tmp0);
    let mut tmp6 = r.x;
    tmp6.add_inp(&tmp4);
    let mut tmp5 = tmp4;
    tmp5.square_inp();
    let mut zsquared = r.z;
    zsquared.square_inp();
    r.x = tmp5;
    r.x.sub_inp(&tmp3);
    r.x.sub_inp(&tmp3);
    r.z.add_inp(&r.y);
    r.z.square_inp();
    r.z.sub_inp(&tmp1);
    r.z.sub_inp(&zsquared);
    r.y = tmp3;
    r.y.sub_inp(&r.x);
    r.y.mul_inp(&tmp4);
    tmp2.double_inp();
    tmp2.double_inp();
    tmp2.double_inp();
    r.y.sub_inp(&tmp2);
    let mut tmp3 = tmp4;
    tmp3.mul_inp(&zsquared);
    tmp3.double_inp();
    let tmp3 = -tmp3;
    tmp6.square_inp();
    tmp6.sub_inp(&tmp0);
    tmp6.sub_inp(&tmp5);
    tmp1.double_inp();
    tmp1.double_inp();
    tmp6.sub_inp(&tmp1);
    let mut tmp0 = r.z;
    tmp0.mul_inp(&zsquared);
    tmp0.double_inp();

    (tmp0, tmp3, tmp6)
}

#[cfg(not(target_vendor = "succinct"))]
fn doubling_step(r: &mut G2Projective) -> (Fp2, Fp2, Fp2) {
    // Adaptation of Algorithm 26, https://eprint.iacr.org/2010/354.pdf
    let tmp0 = r.x.square();
    let tmp1 = r.y.square();
    let tmp2 = tmp1.square();
    let tmp3 = (tmp1 + r.x).square() - tmp0 - tmp2;
    let tmp3 = tmp3 + tmp3;
    let tmp4 = tmp0 + tmp0 + tmp0;
    let tmp6 = r.x + tmp4;
    let tmp5 = tmp4.square();
    let zsquared = r.z.square();
    r.x = tmp5 - tmp3 - tmp3;
    r.z = (r.z + r.y).square() - tmp1 - zsquared;
    r.y = (tmp3 - r.x) * tmp4;
    let tmp2 = tmp2 + tmp2;
    let tmp2 = tmp2 + tmp2;
    let tmp2 = tmp2 + tmp2;
    r.y -= tmp2;
    let tmp3 = tmp4 * zsquared;
    let tmp3 = tmp3 + tmp3;
    let tmp3 = -tmp3;
    let tmp6 = tmp6.square() - tmp0 - tmp5;
    let tmp1 = tmp1 + tmp1;
    let tmp1 = tmp1 + tmp1;
    let tmp6 = tmp6 - tmp1;
    let tmp0 = r.z * zsquared;
    let tmp0 = tmp0 + tmp0;

    (tmp0, tmp3, tmp6)
}

#[cfg(target_vendor = "succinct")]
fn addition_step(r: &mut G2Projective, q: &G2Affine) -> (Fp2, Fp2, Fp2) {
    // Adaptation of Algorithm 27, https://eprint.iacr.org/2010/354.pdf
    let mut zsquared = r.z;
    zsquared.square_inp();
    let mut ysquared = q.y;
    ysquared.square_inp();
    let mut t0 = q.y;
    t0.add_inp(&r.z);
    t0.square_inp();
    t0.sub_inp(&ysquared);
    t0.sub_inp(&zsquared);
    t0.mul_inp(&zsquared);
    let mut t1 = zsquared;
    t1.mul_inp(&q.x);
    t1.sub_inp(&r.x);
    let mut t2 = t1;
    t2.square_inp();
    let mut t3 = t2;
    t3.double_inp();
    t3.double_inp();
    let mut t4 = t3;
    t4.mul_inp(&t1);
    t0.sub_inp(&r.y);
    t0.sub_inp(&r.y);
    let mut t5 = t0;
    t5.mul_inp(&q.x);
    t3.mul_inp(&r.x);
    r.x = t0;
    r.x.square_inp();
    r.x.sub_inp(&t4);
    r.x.sub_inp(&t3);
    r.x.sub_inp(&t3);
    r.z.add_inp(&t1);
    r.z.square_inp();
    r.z.sub_inp(&zsquared);
    r.z.sub_inp(&t2);
    let mut t6 = q.y;
    t6.add_inp(&r.z);
    t3.sub_inp(&r.x);
    t3.mul_inp(&t0);
    t4.mul_inp(&r.y);
    t4.double_inp();
    r.y = t3;
    r.y.sub_inp(&t4);
    t6.square_inp();
    t6.sub_inp(&ysquared);
    let mut ztsquared = r.z;
    ztsquared.square_inp();
    t6.sub_inp(&ztsquared);
    t5.double_inp();
    t5.sub_inp(&t6);
    let mut t6 = r.z;
    t6.double_inp();
    let mut t0 = -t0;
    t0.double_inp();

    (t6, t0, t5)
}

#[cfg(not(target_vendor = "succinct"))]
fn addition_step(r: &mut G2Projective, q: &G2Affine) -> (Fp2, Fp2, Fp2) {
    // Adaptation of Algorithm 27, https://eprint.iacr.org/2010/354.pdf
    let zsquared = r.z.square();
    let ysquared = q.y.square();
    let t0 = zsquared * q.x;
    let t1 = ((q.y + r.z).square() - ysquared - zsquared) * zsquared;
    let t2 = t0 - r.x;
    let t3 = t2.square();
    let t4 = t3 + t3;
    let t4 = t4 + t4;
    let t5 = t4 * t2;
    let t6 = t1 - r.y - r.y;
    let t9 = t6 * q.x;
    let t7 = t4 * r.x;
    r.x = t6.square() - t5 - t7 - t7;
    r.z = (r.z + t2).square() - zsquared - t3;
    let t10 = q.y + r.z;
    let t8 = (t7 - r.x) * t6;
    let t0 = r.y * t5;
    let t0 = t0 + t0;
    r.y = t8 - t0;
    let t10 = t10.square() - ysquared;
    let ztsquared = r.z.square();
    let t10 = t10 - ztsquared;
    let t9 = t9 + t9 - t10;
    let t10 = r.z + r.z;
    let t6 = -t6;
    let t1 = t6 + t6;

    (t10, t1, t9)
}

impl PairingCurveAffine for G1Affine {
    type Pair = G2Affine;
    type PairingResult = Gt;

    fn pairing_with(&self, other: &Self::Pair) -> Self::PairingResult {
        pairing(self, other)
    }
}

impl PairingCurveAffine for G2Affine {
    type Pair = G1Affine;
    type PairingResult = Gt;

    fn pairing_with(&self, other: &Self::Pair) -> Self::PairingResult {
        pairing(other, self)
    }
}

/// A [`pairing::Engine`] for BLS12-381 pairing operations.
#[cfg_attr(docsrs, doc(cfg(feature = "pairings")))]
#[derive(Clone, Debug)]
pub struct Bls12;

impl Engine for Bls12 {
    type Fr = Scalar;
    type G1 = G1Projective;
    type G1Affine = G1Affine;
    type G2 = G2Projective;
    type G2Affine = G2Affine;
    type Gt = Gt;

    fn pairing(p: &Self::G1Affine, q: &Self::G2Affine) -> Self::Gt {
        pairing(p, q)
    }
}

impl pairing::MillerLoopResult for MillerLoopResult {
    type Gt = Gt;

    fn final_exponentiation(&self) -> Self::Gt {
        self.final_exponentiation()
    }
}

#[cfg(feature = "alloc")]
impl MultiMillerLoop for Bls12 {
    type G2Prepared = G2Prepared;
    type Result = MillerLoopResult;

    fn multi_miller_loop(terms: &[(&Self::G1Affine, &Self::G2Prepared)]) -> Self::Result {
        multi_miller_loop(terms)
    }
}

#[test]
fn test_gt_generator() {
    assert_eq!(
        Gt::generator(),
        pairing(&G1Affine::generator(), &G2Affine::generator())
    );
}

#[test]
fn test_bilinearity() {
    use crate::Scalar;

    let a = Scalar::from_raw([1, 2, 3, 4]).invert().unwrap().square();
    let b = Scalar::from_raw([5, 6, 7, 8]).invert().unwrap().square();
    let c = a * b;

    let g = G1Affine::from(G1Affine::generator() * a);
    let h = G2Affine::from(G2Affine::generator() * b);
    let p = pairing(&g, &h);

    assert!(p != Gt::identity());

    let expected = G1Affine::from(G1Affine::generator() * c);

    assert_eq!(p, pairing(&expected, &G2Affine::generator()));
    assert_eq!(
        p,
        pairing(&G1Affine::generator(), &G2Affine::generator()) * c
    );
}

#[test]
fn test_unitary() {
    let g = G1Affine::generator();
    let h = G2Affine::generator();
    let p = -pairing(&g, &h);
    let q = pairing(&g, &-h);
    let r = pairing(&-g, &h);

    assert_eq!(p, q);
    assert_eq!(q, r);
}

#[cfg(feature = "alloc")]
#[test]
fn test_multi_miller_loop() {
    let a1 = G1Affine::generator();
    let b1 = G2Affine::generator();

    let a2 = G1Affine::from(
        G1Affine::generator() * Scalar::from_raw([1, 2, 3, 4]).invert().unwrap().square(),
    );
    let b2 = G2Affine::from(
        G2Affine::generator() * Scalar::from_raw([4, 2, 2, 4]).invert().unwrap().square(),
    );

    let a3 = G1Affine::identity();
    let b3 = G2Affine::from(
        G2Affine::generator() * Scalar::from_raw([9, 2, 2, 4]).invert().unwrap().square(),
    );

    let a4 = G1Affine::from(
        G1Affine::generator() * Scalar::from_raw([5, 5, 5, 5]).invert().unwrap().square(),
    );
    let b4 = G2Affine::identity();

    let a5 = G1Affine::from(
        G1Affine::generator() * Scalar::from_raw([323, 32, 3, 1]).invert().unwrap().square(),
    );
    let b5 = G2Affine::from(
        G2Affine::generator() * Scalar::from_raw([4, 2, 2, 9099]).invert().unwrap().square(),
    );

    let b1_prepared = G2Prepared::from(b1);
    let b2_prepared = G2Prepared::from(b2);
    let b3_prepared = G2Prepared::from(b3);
    let b4_prepared = G2Prepared::from(b4);
    let b5_prepared = G2Prepared::from(b5);

    let expected = pairing(&a1, &b1)
        + pairing(&a2, &b2)
        + pairing(&a3, &b3)
        + pairing(&a4, &b4)
        + pairing(&a5, &b5);

    let test = multi_miller_loop(&[
        (&a1, &b1_prepared),
        (&a2, &b2_prepared),
        (&a3, &b3_prepared),
        (&a4, &b4_prepared),
        (&a5, &b5_prepared),
    ])
    .final_exponentiation();

    assert_eq!(expected, test);
}

#[test]
fn test_miller_loop_result_default() {
    assert_eq!(
        MillerLoopResult::default().final_exponentiation(),
        Gt::identity(),
    );
}

#[cfg(feature = "zeroize")]
#[test]
fn test_miller_loop_result_zeroize() {
    use zeroize::Zeroize;

    let mut m = multi_miller_loop(&[
        (&G1Affine::generator(), &G2Affine::generator().into()),
        (&-G1Affine::generator(), &G2Affine::generator().into()),
    ]);
    m.zeroize();
    assert_eq!(m.0, MillerLoopResult::default().0);
}

#[test]
fn tricking_miller_loop_result() {
    assert_eq!(
        multi_miller_loop(&[(&G1Affine::identity(), &G2Affine::generator().into())]).0,
        Fp12::one()
    );
    assert_eq!(
        multi_miller_loop(&[(&G1Affine::generator(), &G2Affine::identity().into())]).0,
        Fp12::one()
    );
    assert_ne!(
        multi_miller_loop(&[
            (&G1Affine::generator(), &G2Affine::generator().into()),
            (&-G1Affine::generator(), &G2Affine::generator().into())
        ])
        .0,
        Fp12::one()
    );
    assert_eq!(
        multi_miller_loop(&[
            (&G1Affine::generator(), &G2Affine::generator().into()),
            (&-G1Affine::generator(), &G2Affine::generator().into())
        ])
        .final_exponentiation(),
        Gt::identity()
    );
}
