use mesh::mesh::Mesh;
use sine::sine::Sine;
use dac::dac::Dac;
use add::add::Add;
use mult::mult::Mult;
use constant::constant::Constant;

#[test]
fn prompt() {
    let mut mesh = Mesh::new();
    let stream = mesh.run();
    loop {
        mesh.prompt();
    }
}


//#[test]
fn io() {
    let mut mesh = Mesh::new();
    let stream = mesh.run();
    let mut sum = 0;
    mesh.new_processor(Box::new(Constant::new())); //0
    mesh.new_processor(Box::new(Constant::new())); //1
    mesh.new_processor(Box::new(Constant::new())); //2
    mesh.new_processor(Box::new(Sine::new()));     //3
    mesh.new_processor(Box::new(Sine::new()));     //4
    mesh.new_processor(Box::new(Mult::new()));     //5
    mesh.new_processor(Box::new(Add::new()));      //6 
    mesh.new_processor(Box::new(Dac::new()));      //7
    mesh.set_constant(0, 442.1);
    mesh.set_constant(1, 1042.6);
    mesh.set_constant(2, 888.8);
    mesh.new_connection(0, 0, 6, 0);
    mesh.new_connection(1, 0, 5, 0);
    mesh.new_connection(2, 0, 3, 0);
    mesh.new_connection(3, 0, 5, 1);
    mesh.new_connection(5, 0, 6, 1);
    mesh.new_connection(6, 0, 4, 0);
    mesh.new_connection(4, 0, 7, 0);

    loop {}
}
