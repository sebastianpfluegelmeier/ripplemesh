use mesh::mesh::Mesh;
use sine::sine::Sine;
use dac::dac::Dac;
use constant::constant::Constant;

#[test]
fn io() {
    let mut mesh = Mesh::new();
    let stream = mesh.run();
    let mut sum = 0;
//    for i in 0 .. 100{
//        for j in 0 .. 1000 {
//            for k in 0 .. 10 {
//                sum += 1;
//            }
//        }
//    }
//    println!("sum: {}", sum);
    mesh.new_processor(Box::new(Constant::new()));
    mesh.new_processor(Box::new(Sine::new()));
    mesh.new_processor(Box::new(Dac::new()));
    mesh.set_constant(0, 666.6);
    mesh.new_connection(0, 0, 1, 0);
    mesh.new_connection(1, 0, 2, 0);
    loop {}
}
