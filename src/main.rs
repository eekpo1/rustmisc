use crate::paras::RustPars;

mod paras;

fn main() {
    println!("Hello, world!");

    let mut a = vec![1, 2, 3, 4];
    let mut b: Vec<i32> = (1..500).collect();

    println!("{:?}", a);

    println!("{:?}", a.chunks(1).into_iter().map(|x| x).collect::<Vec<&[i32]>>());

    a = a.into_iter().clone().map(|x| x + 1).collect::<Vec<i32>>();
    println!("{:?}", a);

    let mut pars = RustPars::new(4);
    pars.map_parallel(b.clone(), |a| Some(a + 1)).drop();

}
