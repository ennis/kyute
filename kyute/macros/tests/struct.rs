#![feature(specialization)]
use kyute::model::Lens;
use kyute_macros::Data;

#[derive(Clone, Data)]
struct Fields {
    a: i32,
    b: f32,
    c: String,
}

#[derive(Clone, Data)]
struct Tuple(i32, f32, String);

#[test]
fn struct_check() {
    let mut fields = Fields {
        a: 1,
        b: 2.0,
        c: "third".to_string(),
    };

    let mut tup = Tuple(4, 5.0, "sixth".to_string());

    assert_eq!(Fields::a.get(&fields), &1);
    assert_eq!(Fields::b.get(&fields), &2.0);
    assert_eq!(Fields::c.get(&fields), "third");

    assert_eq!(Tuple::elem_0.get(&tup), &4);
    assert_eq!(Tuple::elem_1.get(&tup), &5.0);
    assert_eq!(Tuple::elem_2.get(&tup), "sixth");

    *Fields::c.get_mut(&mut fields) = "seventh".to_string();
    *Tuple::elem_2.get_mut(&mut tup) = "eighth".to_string();

    assert_eq!(Fields::c.get(&fields), "seventh");
    assert_eq!(Tuple::elem_2.get(&tup), "eighth");
}
