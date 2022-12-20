use crate::optimizer::ir_constructor::{
    basic_block, compare_functions, function, get_func, init, inst, Opcode,
};
use ctl::optimizer::ir::function::Function;
use ctl::optimizer::ir::inst::{Cc, InstData, InstId};
use ctl::optimizer::ir_builder::build_intermediate_representation;

/// Tests on building the basic blocks from the linear IR

#[test]
fn build_empty_function() -> Result<(), String> {
    let mut func = Function::new("".to_string());

    // Linear IR
    func.create_inst(InstData::ReturnVoid);

    build_intermediate_representation(&mut func);

    // Constructing the graph manually
    function(
        init(1, 1),
        &[basic_block(0).insts(&[inst(0, Opcode::ReturnVoid)])],
    );

    // Comparing of what is built with what is constructed manually
    compare_functions(&func, get_func())
}

#[test]
fn build_function_2_parameters() -> Result<(), String> {
    let mut func = Function::new("".to_string());

    // Linear IR
    func.create_inst(InstData::Parameter);
    func.create_inst(InstData::Parameter);
    func.create_inst(InstData::ReturnVoid);

    build_intermediate_representation(&mut func);

    // Constructing the graph manually
    function(
        init(3, 1),
        &[basic_block(0).insts(&[
            inst(0, Opcode::Parameter),
            inst(1, Opcode::Parameter),
            inst(2, Opcode::ReturnVoid),
        ])],
    );

    // Comparing of what is built with what is constructed manually
    compare_functions(&func, get_func())
}

#[test]
fn build_function_returning_param() -> Result<(), String> {
    let mut func = Function::new("".to_string());

    // Linear IR
    func.create_inst(InstData::Parameter);
    func.create_inst(InstData::Return(InstId(0)));

    build_intermediate_representation(&mut func);

    // Constructing the graph manually
    function(
        init(3, 1),
        &[basic_block(0).insts(&[
            inst(0, Opcode::Parameter),
            inst(1, Opcode::Return).inputs(&[0]),
        ])],
    );

    // Comparing of what is built with what is constructed manually
    compare_functions(&func, get_func())
}

#[test]
fn build_function_returning_param_plus_local() -> Result<(), String> {
    let mut func = Function::new("".to_string());

    // Linear IR
    func.create_inst(InstData::Parameter);
    func.create_inst(InstData::Alloc);
    func.create_inst(InstData::Constant(1));
    func.create_inst(InstData::Store(InstId(2), InstId(1)));
    func.create_inst(InstData::Load(InstId(1)));
    func.create_inst(InstData::Add(InstId(0), InstId(4)));
    func.create_inst(InstData::Return(InstId(5)));

    build_intermediate_representation(&mut func);

    // Constructing the graph manually
    function(
        init(7, 1),
        &[basic_block(0).insts(&[
            inst(0, Opcode::Parameter),
            inst(1, Opcode::Alloc),
            inst(2, Opcode::Constant).value(1),
            inst(3, Opcode::Store).inputs(&[2]).dest(1),
            inst(4, Opcode::Load).inputs(&[1]),
            inst(5, Opcode::Add).inputs(&[0, 4]),
            inst(6, Opcode::Return).inputs(&[5]),
        ])],
    );

    // Comparing of what is built with what is constructed manually
    compare_functions(&func, get_func())
}

/// fn main(p: i64) -> i64 {
///     return (p - 2) * 4 / 2 % 3;
/// }
#[test]
fn build_arithmetic_expression() -> Result<(), String> {
    let mut func = Function::new("".to_string());

    // Linear IR
    func.create_inst(InstData::Parameter);
    func.create_inst(InstData::Constant(2));
    func.create_inst(InstData::Sub(InstId(0), InstId(1)));
    func.create_inst(InstData::Constant(4));
    func.create_inst(InstData::Mul(InstId(2), InstId(3)));
    func.create_inst(InstData::Div(InstId(4), InstId(1)));
    func.create_inst(InstData::Constant(3));
    func.create_inst(InstData::Mod(InstId(5), InstId(6)));
    func.create_inst(InstData::Return(InstId(7)));

    build_intermediate_representation(&mut func);

    // Constructing the graph manually
    function(
        init(9, 1),
        &[basic_block(0).insts(&[
            inst(0, Opcode::Parameter),
            inst(1, Opcode::Constant).value(2),
            inst(2, Opcode::Sub).inputs(&[0, 1]),
            inst(3, Opcode::Constant).value(4),
            inst(4, Opcode::Mul).inputs(&[2, 3]),
            inst(5, Opcode::Div).inputs(&[4, 1]),
            inst(6, Opcode::Constant).value(3),
            inst(7, Opcode::Mod).inputs(&[5, 6]),
            inst(8, Opcode::Return).inputs(&[7]),
        ])],
    );

    // Comparing of what is built with what is constructed manually
    compare_functions(&func, get_func())
}

/// fn main(p: i64) -> i64 {
///     let mut a: i64 = 0;
///     if (p == 0) {
///         a = 1;
///     }
/// }
#[test]
fn build_conditional_branch_with_assign() -> Result<(), String> {
    let mut func = Function::new("".to_string());

    // Linear IR
    func.create_inst(InstData::Parameter);
    func.create_inst(InstData::Alloc);
    func.create_inst(InstData::Constant(0));
    func.create_inst(InstData::Store(InstId(2), InstId(1)));
    func.create_inst(InstData::IfFalse(InstId(0), InstId(2), Cc::Eq, InstId(7)));
    func.create_inst(InstData::Constant(1));
    func.create_inst(InstData::Store(InstId(5), InstId(1)));
    func.create_inst(InstData::ReturnVoid);

    build_intermediate_representation(&mut func);

    // Constructing the graph manually
    function(
        init(9, 3),
        &[
            basic_block(0).succs(&[1, 2]).insts(&[
                inst(0, Opcode::Parameter),
                inst(1, Opcode::Alloc),
                inst(2, Opcode::Constant).value(0),
                inst(3, Opcode::Store).inputs(&[2]).dest(1),
                inst(4, Opcode::Branch).inputs(&[0, 2]).cc(Cc::Eq),
            ]),
            basic_block(1).succs(&[2]).insts(&[
                inst(5, Opcode::Constant).value(1),
                inst(6, Opcode::Store).inputs(&[5]).dest(1),
                inst(8, Opcode::Jump),
            ]),
            basic_block(2).insts(&[inst(7, Opcode::ReturnVoid)]),
        ],
    );

    // Comparing of what is built with what is constructed manually
    compare_functions(&func, get_func())
}

/// fn main(p: i64) -> i64 {
///     if (p == 0) {
///         return 0;
///     } else {
///         return 1;
///     }
/// }
#[test]
fn build_conditional_branch_with_returns() -> Result<(), String> {
    let mut func = Function::new("".to_string());

    // Linear IR
    func.create_inst(InstData::Parameter);
    func.create_inst(InstData::Constant(0));
    func.create_inst(InstData::IfFalse(InstId(0), InstId(1), Cc::Eq, InstId(5)));
    func.create_inst(InstData::Return(InstId(1)));
    func.create_inst(InstData::Goto(InstId(7)));
    func.create_inst(InstData::Constant(1));
    func.create_inst(InstData::Return(InstId(5)));
    func.create_inst(InstData::ReturnVoid);

    build_intermediate_representation(&mut func);

    // Constructing the graph manually
    function(
        init(9, 4),
        &[
            basic_block(0).succs(&[1, 2]).insts(&[
                inst(0, Opcode::Parameter),
                inst(1, Opcode::Constant).value(0),
                inst(2, Opcode::Branch).inputs(&[0, 1]).cc(Cc::Eq),
            ]),
            basic_block(1)
                .succs(&[3])
                .insts(&[inst(3, Opcode::Return).inputs(&[1]), inst(4, Opcode::Jump)]),
            basic_block(2).succs(&[3]).insts(&[
                inst(5, Opcode::Constant).value(1),
                inst(6, Opcode::Return).inputs(&[5]),
                inst(8, Opcode::Jump),
            ]),
            basic_block(3).insts(&[inst(7, Opcode::ReturnVoid)]),
        ],
    );

    // Comparing of what is built with what is constructed manually
    compare_functions(&func, get_func())
}

/// Input code:
/// fn main() {
///     let mut a: i64 = 0;
///     let mut b: i64 = 128;

///     while (a < 8) {
///         a = a + 1;

///         if (a == 3) {
///             continue;
///         }

///         while (b > 0) {
///             b = b - 1;

///             if (b == 4) {
///                 continue;
///             }
///         }

///     }
/// }
#[test]
fn build_conditional_nested_loops_with_continues() -> Result<(), String> {
    let mut func = Function::new("".to_string());

    // Linear IR

    // let mut a: i64 = 0;
    // let mut b: i64 = 128;
    func.create_inst(InstData::Alloc);
    func.create_inst(InstData::Constant(0));
    func.create_inst(InstData::Store(InstId(1), InstId(0)));
    func.create_inst(InstData::Alloc);
    func.create_inst(InstData::Constant(128));
    func.create_inst(InstData::Store(InstId(4), InstId(3)));

    // while (a < 8) {
    func.create_inst(InstData::Load(InstId(0)));
    func.create_inst(InstData::Constant(8));
    func.create_inst(InstData::IfFalse(InstId(6), InstId(7), Cc::Lt, InstId(28)));

    //     a = a + 1;
    func.create_inst(InstData::Load(InstId(0)));
    func.create_inst(InstData::Constant(1));
    func.create_inst(InstData::Add(InstId(9), InstId(10)));
    func.create_inst(InstData::Store(InstId(11), InstId(0)));

    // if (a == 3) {
    //     continue;
    // }
    func.create_inst(InstData::Load(InstId(0)));
    func.create_inst(InstData::Constant(3));
    func.create_inst(InstData::IfFalse(
        InstId(13),
        InstId(14),
        Cc::Eq,
        InstId(17),
    ));
    func.create_inst(InstData::Goto(InstId(6)));

    // Inner loop
    // while (b > 0) {
    //     b = b - 1;
    func.create_inst(InstData::Load(InstId(3)));
    func.create_inst(InstData::IfFalse(InstId(17), InstId(1), Cc::Gt, InstId(27)));
    func.create_inst(InstData::Load(InstId(3)));
    func.create_inst(InstData::Sub(InstId(19), InstId(10)));
    func.create_inst(InstData::Store(InstId(20), InstId(3)));

    // if (b == 4) {
    //     continue;
    // }
    func.create_inst(InstData::Load(InstId(3)));
    func.create_inst(InstData::Constant(4));
    func.create_inst(InstData::IfFalse(
        InstId(22),
        InstId(23),
        Cc::Eq,
        InstId(26),
    ));
    func.create_inst(InstData::Goto(InstId(17)));

    // Go back to the comparison of the inner loop
    func.create_inst(InstData::Goto(InstId(17)));

    // Go back to the comparison of the outer loop
    func.create_inst(InstData::Goto(InstId(6)));

    func.create_inst(InstData::ReturnVoid);

    build_intermediate_representation(&mut func);

    // Constructing the graph manually
    function(
        init(30, 10),
        &[
            basic_block(0).succs(&[1]).insts(&[
                inst(0, Opcode::Alloc),
                inst(1, Opcode::Constant).value(0),
                inst(2, Opcode::Store).inputs(&[1]).dest(0),
                inst(3, Opcode::Alloc),
                inst(4, Opcode::Constant).value(128),
                inst(5, Opcode::Store).inputs(&[4]).dest(3),
                inst(29, Opcode::Jump),
            ]),
            basic_block(1).succs(&[2, 9]).insts(&[
                inst(6, Opcode::Load).inputs(&[0]),
                inst(7, Opcode::Constant).value(8),
                inst(8, Opcode::Branch).inputs(&[6, 7]).cc(Cc::Lt),
            ]),
            basic_block(2).succs(&[3, 4]).insts(&[
                inst(9, Opcode::Load).inputs(&[0]),
                inst(10, Opcode::Constant).value(1),
                inst(11, Opcode::Add).inputs(&[9, 10]),
                inst(12, Opcode::Store).inputs(&[11]).dest(0),
                inst(13, Opcode::Load).inputs(&[0]),
                inst(14, Opcode::Constant).value(3),
                inst(15, Opcode::Branch).inputs(&[13, 14]).cc(Cc::Eq),
            ]),
            basic_block(3).succs(&[1]).insts(&[inst(16, Opcode::Jump)]),
            basic_block(4).succs(&[5, 8]).insts(&[
                inst(17, Opcode::Load).inputs(&[3]),
                inst(18, Opcode::Branch).inputs(&[17, 1]).cc(Cc::Gt),
            ]),
            basic_block(5).succs(&[6, 7]).insts(&[
                inst(19, Opcode::Load).inputs(&[3]),
                inst(20, Opcode::Sub).inputs(&[19, 10]),
                inst(21, Opcode::Store).inputs(&[20]).dest(3),
                inst(22, Opcode::Load).inputs(&[3]),
                inst(23, Opcode::Constant).value(4),
                inst(24, Opcode::Branch).inputs(&[22, 23]).cc(Cc::Eq),
            ]),
            basic_block(6).succs(&[4]).insts(&[inst(25, Opcode::Jump)]),
            basic_block(7).succs(&[4]).insts(&[inst(26, Opcode::Jump)]),
            basic_block(8).succs(&[1]).insts(&[inst(27, Opcode::Jump)]),
            basic_block(9)
                .succs(&[])
                .insts(&[inst(28, Opcode::ReturnVoid)]),
        ],
    );

    // Comparing of what is built with what is constructed manually
    compare_functions(&func, get_func())
}
