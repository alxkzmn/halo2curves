# Removing cyclic curve trait bounds

`CurveExt`, `CurveAffine`, and `Coordinates` form a mutually recursive
declaration group that Aeneas rejects during translation to Lean.

- Replace `CurveExt::AffineExt`'s full `CurveAffine` bound with explicit
  prime-curve and operation bounds, removing the dependency on `CurveAffine`.
  Keep the curve/scalar relationships and make the existing scalar `Ord`
  requirement explicit.
- Move the affine `Base` association into `CurveAffineCoordinates`, which
  `Coordinates<C>` can use without depending on the full `CurveAffine` trait.
  Update the shared curve macro to implement the new trait.
- Add coordinate regression tests for all 14 curves, covering validation,
  identity, accessors, conversions, conditional selection, and layout.

Generic callers using extended affine methods through `CurveExt::AffineExt`
must now require `CurveAffine` explicitly. Qualified affine `Base` references
move to `CurveAffineCoordinates`. Curve arithmetic and coordinate validation
are unchanged.
