//! Regression coverage shared with the pristine published 0.9.0 source.
//! In particular, these tests retain its actual identity-coordinate behavior.

use core::mem::{align_of, size_of};
use ff::Field;
use group::{prime::PrimeCurveAffine, Group};
use halo2curves::{Coordinates, CurveAffine, CurveExt};
use subtle::{Choice, ConditionallySelectable};

// Exercise the extended correspondence at real generic consumers, rather than
// relying on an implicit reverse bound on CurveExt::AffineExt. There is no
// equality between the projective and affine Base associated types here.
fn check_extended_correspondence<C>(point: C)
where
    C: CurveExt,
    C::AffineExt: CurveAffine<CurveExt = C, ScalarExt = C::ScalarExt>,
{
    let affine: C::AffineExt = point.into();
    let prime_affine: <C as group::prime::PrimeCurve>::Affine = affine;
    let from_prime: C = prime_affine.to_curve();
    let from_extended: C = affine.into();
    assert_eq!(from_prime, point);
    assert_eq!(from_extended, point);
    assert_eq!(point.to_affine(), affine);

    let scalar = C::ScalarExt::from(7);
    let owned_product: C = affine * scalar;
    let borrowed_product: C = affine * &scalar;
    assert_eq!(owned_product, point * scalar);
    assert_eq!(borrowed_product, owned_product);

    let sum: C = affine + affine;
    let difference: C = affine - affine;
    assert_eq!(sum, point.double());
    assert_eq!(difference, C::identity());
}

fn check_coordinates<A: CurveAffine>() {
    // The representation remains two base-field values, tied to the affine
    // curve by the generic parameter rather than interchangeable field data.
    assert_eq!(size_of::<Coordinates<A>>(), size_of::<(A::Base, A::Base)>());
    assert_eq!(
        align_of::<Coordinates<A>>(),
        align_of::<(A::Base, A::Base)>()
    );

    let generator = A::generator();
    let doubled = A::from(generator.to_curve().double());
    let points = [generator, doubled, -generator];
    for point in points {
        assert!(!bool::from(point.is_identity()));
        let coordinates = point.coordinates().unwrap();
        let x: &A::Base = coordinates.x();
        let y: &A::Base = coordinates.y();
        let u: &A::Base = coordinates.u();
        let v: &A::Base = coordinates.v();
        assert!(core::ptr::eq(x, u));
        assert!(core::ptr::eq(y, v));
        assert_eq!(y.square(), (x.square() + A::a()) * x + A::b());

        let recovered = A::from_xy(*x, *y).unwrap();
        assert_eq!(recovered, point);
        let validated = Coordinates::<A>::from_xy(*x, *y).unwrap();
        assert_eq!(validated.x(), x);
        assert_eq!(validated.y(), y);
        check_extended_correspondence(point.to_curve());
    }

    // Find a nonidentity pair that independently fails y^2 = x^3 + ax + b.
    // At most two of these three distinct nonzero y-values can be roots.
    let x = A::Base::ONE;
    let one = A::Base::ONE;
    let y = [one, one + one, one + one + one]
        .into_iter()
        .find(|y| y.square() != (x.square() + A::a()) * x + A::b())
        .expect("one of three candidate y-values must be off the curve");
    assert_ne!(x, A::Base::ZERO);
    assert_ne!(y, A::Base::ZERO);
    assert!(bool::from(A::from_xy(x, y).is_none()));
    assert!(bool::from(Coordinates::<A>::from_xy(x, y).is_none()));

    // Upstream's macro recognizes (0,0) as identity and accepts it in from_xy.
    // Its coordinates() therefore returns Some(0,0), despite the trait comment.
    let identity = A::identity();
    let identity_coordinates = identity.coordinates().unwrap();
    assert_eq!(*identity_coordinates.x(), A::Base::ZERO);
    assert_eq!(*identity_coordinates.y(), A::Base::ZERO);
    assert_eq!(A::from_xy(A::Base::ZERO, A::Base::ZERO).unwrap(), identity);
    let validated_identity = Coordinates::<A>::from_xy(A::Base::ZERO, A::Base::ZERO).unwrap();
    assert_eq!(validated_identity.x(), identity_coordinates.x());
    assert_eq!(validated_identity.y(), identity_coordinates.y());
    assert_eq!(A::from(identity.to_curve()), identity);

    let first = generator.coordinates().unwrap();
    let second = doubled.coordinates().unwrap();
    assert!(first.x() != second.x() || first.y() != second.y());
    for (choice, expected, expected_point) in [(0, first, generator), (1, second, doubled)] {
        let selected = Coordinates::conditional_select(&first, &second, Choice::from(choice));
        assert_eq!(selected.x(), expected.x());
        assert_eq!(selected.y(), expected.y());
        assert_eq!(
            A::from_xy(*selected.x(), *selected.y()).unwrap(),
            expected_point
        );
    }
}

macro_rules! coordinate_regression {
    ($name:ident, $affine:ty) => {
        #[test]
        fn $name() {
            check_coordinates::<$affine>();
        }
    };
}

coordinate_regression!(bls12381_g1, halo2curves::bls12381::G1Affine);
coordinate_regression!(bls12381_g2, halo2curves::bls12381::G2Affine);
coordinate_regression!(bn256_g1, halo2curves::bn256::G1Affine);
coordinate_regression!(bn256_g2, halo2curves::bn256::G2Affine);
coordinate_regression!(grumpkin, halo2curves::grumpkin::G1Affine);
coordinate_regression!(pallas, halo2curves::pasta::PallasAffine);
coordinate_regression!(vesta, halo2curves::pasta::VestaAffine);
coordinate_regression!(pluto_g1, halo2curves::pluto_eris::G1Affine);
coordinate_regression!(eris, halo2curves::pluto_eris::ErisAffine);
coordinate_regression!(pluto_g2, halo2curves::pluto_eris::G2Affine);
coordinate_regression!(secp256k1, halo2curves::secp256k1::Secp256k1Affine);
coordinate_regression!(secp256r1, halo2curves::secp256r1::Secp256r1Affine);
coordinate_regression!(secq256k1, halo2curves::secq256k1::Secq256k1Affine);
coordinate_regression!(t256, halo2curves::t256::T256Affine);
