#![allow(unused_imports)]
use faer::modules::core as faer_core;
use faer::prelude::*;
use faer::{complex_native::c64, mat, ColRef, Mat, MatMut, MatRef};

// error[E0277]: the trait bound `DenseCol: MatAdd<Dense>` is not satisfied
//  --> src/main.rs:17:23
//    |
// 17 |     x.as_ref().col(0) + &x;
//    |                       ^ the trait `MatAdd<Dense>` is not implemented for `DenseCol`
//    |
//    = help: the trait `MatAdd<DenseCol>` is implemented for `DenseCol`
//    = help: for that trait implementation, expected `DenseCol`, found `Dense`
//    = note: required for `Matrix<DenseColRef<'_, f64>>` to implement `std::ops::Add<&Matrix<DenseOwn<f64>>>`
//
// fn adding_incompatible_matrices(lhs: MatRef<'_, f64>, rhs: ColRef<'_, f64>) {
//     lhs + rhs
// }

// error[E0515]: cannot return value referencing local variable `m`
//   --> src/main.rs:31:5
//    |
// 31 |     m.as_ref()
//    |     -^^^^^^^^^
//    |     |
//    |     returns a value referencing data owned by the current function
//    |     `m` is borrowed here
//
fn returning_dangling_reference<'a>(lhs: MatRef<'a, f64>, rhs: MatRef<'a, f64>) -> MatRef<'a, f64> {
    let mut m = lhs + rhs;
    m.as_ref()
}

// error[E0502]: cannot borrow `dst` as mutable because it is also borrowed as immutable
//   --> src/main.rs:47:9
//    |
// 47 |     let a = dst.as_ref();
//    |             --- immutable borrow occurs here
// 48 |     faer_core::mul::matmul(
// 49 |         dst.as_mut(),
//    |         ^^^^^^^^^^^^ mutable borrow occurs here
// 50 |         a,
//    |         - immutable borrow later used here
fn breaking_no_alias_guarantees(mut dst: MatMut<'_, c64>) {
    let a = dst.as_ref();
    faer_core::mul::matmul(
        dst.as_mut(),
        a,
        a.adjoint(),
        None,
        c64::new(1.0, 0.0),
        faer::Parallelism::None,
    );
}

fn dense_example() {
    println!("\n\ndense example");

    let a = Mat::from_fn(4, 4, |i, j| i as f64 + j as f64);
    let b = mat![[1.0], [2.0], [3.0], [4.0]];

    // solving a linear system (selfadjoint)
    let lblt = a.lblt(faer::Side::Lower);
    let x = lblt.solve(&b);
    dbg!((&a * &x - &b).norm_l2());

    // computing eigenvalues
    let complex_eigenvalues = a.eigenvalues::<c64>();
    let real_eigenvalues = a.selfadjoint_eigenvalues(faer::Side::Lower);

    dbg!(&complex_eigenvalues);
    dbg!(&real_eigenvalues);
}

fn sparse_example() -> Result<(), Box<dyn std::error::Error>> {
    use faer::sparse::*;
    use faer_core::sparse::*;

    println!("\n\nsparse example");

    let mut a = SparseColMat::<usize, f64>::try_new_from_triplets(
        4,
        4,
        &[
            (0, 0, 10.0),
            (1, 1, 20.0),
            (2, 2, 30.0),
            (3, 3, 40.0),
            (0, 1, 3.0),
            (1, 0, 2.0),
            (3, 2, 1.0),
        ],
    )?;

    let b = mat![[1.0], [2.0], [3.0], [4.0]];

    // solving a linear system
    let lu = a.as_ref().sp_lu().unwrap();
    let x = lu.solve(&b);
    dbg!((&a * &x - &b).norm_l2());

    // splitting up the solve into symbolic and numeric parts

    // the symbolic part is reference-counted since it is mostly immutable.
    // this makes it cheap to clone and pass around.
    let lu_symbolic = solvers::SymbolicLu::try_new(a.as_ref().symbolic())?;

    let lu_numeric = solvers::Lu::try_new_with_symbolic(lu_symbolic.clone(), a.as_ref()).unwrap();
    let x = lu_numeric.solve(&b);
    dbg!((&a * &x - &b).norm_l2());

    for value in a.as_mut().values_of_col_mut(1) {
        *value *= 2.0;
    }
    let lu_numeric = solvers::Lu::try_new_with_symbolic(lu_symbolic, (&a + &a).as_ref()).unwrap();
    let x = lu_numeric.solve(&b);
    dbg!(((&a + &a) * &x - &b).norm_l2());

    Ok(())
}

fn error_report() {
    println!("\n\nerror messages");
    assert!(std::panic::catch_unwind(|| {
        let a = Mat::<f64>::zeros(5, 4);
        let b = Mat::<f64>::zeros(4, 5);
        let _ = &a + &b;
    })
    .is_err());
}

fn main() {
    dense_example();
    sparse_example().unwrap();
    error_report();
}
