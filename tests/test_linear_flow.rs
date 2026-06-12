use flat_cont::basicblock::{BasicBlock, BasicBlockGraph, Node as BbNode, NodeId as BbNid, Terminator as BbTerm};
use flat_cont::bb2flatcont::bb_to_flat_cont;
use flat_cont::common::{Opcode, RawType};
use flat_cont::flatcont::{Node as FcNode, NodeId as FcNid, Terminator as FcTerm};

fn fid(x: u32) -> FcNid { FcNid(x) }
fn cid(x: u32) -> flat_cont::basicblock::ContId { flat_cont::basicblock::ContId(x) }

// f(x) { let y = x + 1; return y * 2; }
fn make_linear_graph() -> BasicBlockGraph {
    BasicBlockGraph {
        name: Some("test".into()),
        static_data: vec![],
        externals: vec![],
        nodes: vec![
            BbNode::Param(0),                                                         // 0: x
            BbNode::Const(1, RawType::Scalar(32)),                                   // 1: 1
            BbNode::Compute(Opcode::Add, smallvec::smallvec![BbNid(0), BbNid(1)]),   // 2: y = x + 1
            BbNode::Const(2, RawType::Scalar(32)),                                   // 3: 2
            BbNode::Compute(Opcode::Mul, smallvec::smallvec![BbNid(2), BbNid(3)]),   // 4: ret = y * 2
        ],
        bbs: vec![
            BasicBlock {
                node_ids: vec![BbNid(0), BbNid(1), BbNid(2)],
                terminator: BbTerm::Jump {
                    effect_args: vec![],
                    common_args: vec![],
                    target: cid(1),
                },
            },
            BasicBlock {
                node_ids: vec![BbNid(3), BbNid(4)],
                terminator: BbTerm::Return {
                    effect_args: vec![],
                    common_args: vec![BbNid(4)], // ret = node 4
                },
            },
        ],
    }
}

#[test]
fn test_linear_flow() {
    let graph = make_linear_graph();
    let result = bb_to_flat_cont(&graph);

    assert_eq!(result.continuations.len(), 2);

    // ---- Cont 0: params=[x], body=[Const(1), y=x+1] ----
    let c0 = &result.continuations[0];
    assert_eq!(c0.params.len(), 1);
    assert_eq!(c0.effects.len(), 0);
    assert_eq!(c0.nodes.len(), 2);
    // y=x+1 at body[1]: lhs→param{0}, rhs→body[0]{Const 1}
    assert!(matches!(&c0.nodes[1], FcNode::Compute(Opcode::Add, ops)
        if ops[0] == fid(0) && ops[1] == fid(1)
    ));
    // Jump: passes y{body[1]=fid(2)} to target cont 1
    assert!(matches!(&c0.terminator, FcTerm::Jump { common_args, target, .. }
        if common_args == &vec![fid(2)] && target.0 == 1
    ));

    // ---- Cont 1: params=[y], body=[Const(2), ret=y*2] ----
    let c1 = &result.continuations[1];
    assert_eq!(c1.params.len(), 1);
    assert_eq!(c1.nodes.len(), 2);
    // ret at body[1]: lhs→param{0}(y), rhs→body[0]{Const 2}
    assert!(matches!(&c1.nodes[1], FcNode::Compute(Opcode::Mul, ops)
        if ops[0] == fid(0) && ops[1] == fid(1)
    ));
    // Return: common_args=[ret{body[1]=fid(2)}], inherited y pruned, own ret kept
    assert!(matches!(&c1.terminator, FcTerm::Return { common_args, .. }
        if common_args == &vec![fid(2)]
    ));
}
