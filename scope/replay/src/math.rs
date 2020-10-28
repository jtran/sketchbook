use crate::core::*;

pub fn clamp(s: Scalar) -> Scalar {
    s.max(0.0).min(1.0)
}

pub fn mix_scalar(s1: Scalar, s2: Scalar, percent: Scalar) -> Scalar {
    s1 + percent * (s2 - s1)
}

pub fn quadratic_out(s: Scalar) -> Scalar {
    let s = clamp(s);

    -(s * (s - 2.0))
}
