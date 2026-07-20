use ordofp::fix::Fix;
use ordofp::typeclasses::hkt::{FunctorHKT, HKT};

enum ExprF<A> {
    Val(i32),
    Add(A, A),
}

struct ExprHKT;

impl HKT for ExprHKT {
    type Target<T> = ExprF<T>;
}

impl FunctorHKT for ExprHKT {
    fn map<A, B, F>(fa: ExprF<A>, mut f: F) -> ExprF<B>
    where
        F: FnMut(A) -> B,
    {
        match fa {
            ExprF::Val(i) => ExprF::Val(i),
            ExprF::Add(l, r) => ExprF::Add(f(l), f(r)),
        }
    }
}

type Expr = Fix<ExprHKT>;

#[test]
fn test_eval_cata() {
    let val1 = Expr::new(ExprF::Val(10));
    let val2 = Expr::new(ExprF::Val(20));
    let add = Expr::new(ExprF::Add(val1, val2));

    let eval = |e: ExprF<i32>| -> i32 {
        match e {
            ExprF::Val(i) => i,
            ExprF::Add(l, r) => l + r,
        }
    };

    let result = add.cata(eval);
    assert_eq!(result, 30);
}

#[test]
fn test_list_ana() {
    // ListF A = Nil | Cons i32 A
    enum ListF<A> {
        Nil,
        Cons(i32, A),
    }

    struct ListHKT;
    impl HKT for ListHKT {
        type Target<T> = ListF<T>;
    }
    impl FunctorHKT for ListHKT {
        fn map<A, B, F>(fa: ListF<A>, mut f: F) -> ListF<B>
        where
            F: FnMut(A) -> B,
        {
            match fa {
                ListF::Nil => ListF::Nil,
                ListF::Cons(h, t) => ListF::Cons(h, f(t)),
            }
        }
    }

    let coalg = |n: i32| -> ListF<i32> {
        if n <= 0 {
            ListF::Nil
        } else {
            ListF::Cons(n, n - 1)
        }
    };

    let list: Fix<ListHKT> = Fix::ana(5, coalg);

    // Convert back to vec
    let to_vec = |l: ListF<Vec<i32>>| -> Vec<i32> {
        match l {
            ListF::Nil => vec![],
            ListF::Cons(h, mut t) => {
                let mut v = vec![h];
                v.append(&mut t);
                v
            }
        }
    };

    let result = list.cata(to_vec);
    assert_eq!(result, vec![5, 4, 3, 2, 1]);
}
