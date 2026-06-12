use flat_cont::basicblock::{BasicBlock, BasicBlockGraph, Node as BbNode, NodeId as BbNid, Terminator as BbTerm};
use flat_cont::bb2flatcont::bb_to_flat_cont;
use flat_cont::common::RawType;
use flat_cont::flatcont::{Node as FcNode, NodeId as FcNid, Terminator as FcTerm};

fn fid(x: u32) -> FcNid { FcNid(x) }
fn cid(x: u32) -> flat_cont::basicblock::ContId { flat_cont::basicblock::ContId(x) }

// BB0: EffectParam → Load(effect_param)  ─→  BB1  (passes Load as new effect token)
// BB1: Store(effect=Load)  ─→  Return(effect=Store)
fn make_effect_chain_graph() -> BasicBlockGraph {
    BasicBlockGraph {
        name: Some("effect_chain".into()),
        static_data: vec![],
        externals: vec![],
        nodes: vec![
            BbNode::EffectParam(0),                                                         // 0: initial effect
            BbNode::Const(16, RawType::Ptr),                                                // 1: addr
            BbNode::Load { data_type: RawType::Scalar(32), effect_state: BbNid(0), addr: BbNid(1), signed: false }, // 2: load
            BbNode::Const(42, RawType::Scalar(32)),                                          // 3: value
            BbNode::Store { data_type: RawType::Scalar(32), effect_state: BbNid(2), addr: BbNid(1), value: BbNid(3) }, // 4: store
        ],
        bbs: vec![
            BasicBlock {
                node_ids: vec![BbNid(0), BbNid(1), BbNid(2)],
                terminator: BbTerm::Jump { effect_args: vec![], common_args: vec![], target: cid(1) },
            },
            BasicBlock {
                node_ids: vec![BbNid(3), BbNid(4)],
                // Store produces the final effect; Return terminates it
                terminator: BbTerm::Return { effect_args: vec![(String::new(), BbNid(4))], common_args: vec![] },
            },
        ],
    }
}

#[test]
fn test_effect_chain() {
    let graph = make_effect_chain_graph();
    let result = bb_to_flat_cont(&graph);

    assert_eq!(result.continuations.len(), 2);

    // ---- Cont 0: effects=[EffectParam], body=[Const(addr){fid(1)}, Load{fid(2)}] ----
    let c0 = &result.continuations[0];
    assert_eq!(c0.effects.len(), 1);
    assert_eq!(c0.params.len(), 0);
    assert_eq!(c0.nodes.len(), 2);
    // Load at body[1]{fid=2}: effect_state→effects[0]{fid(0)}, addr→body[0]{fid(1)}
    assert!(matches!(&c0.nodes[1], FcNode::Load { effect_state, addr, .. }
        if *effect_state == fid(0) && *addr == fid(1)
    ));
    // Jump: effect_args=[Load{body[1]=fid(2)}]
    assert!(matches!(&c0.terminator, FcTerm::Jump { effect_args, .. }
        if effect_args.len() == 1 && effect_args[0].1 == fid(2)
    ));

    // ---- Cont 1: effects=[Load], params=[addr], body=[Const(val){fid(2)}, Store{fid(3)}] ----
    let c1 = &result.continuations[1];
    assert_eq!(c1.effects.len(), 1);
    assert_eq!(c1.params.len(), 1); // addr(1) is external → param
    assert_eq!(c1.nodes.len(), 2);
    // Store at body[1]{fid=3}: effect_state→effects[0]{fid(1)}, addr→params[0]{fid(0)}, value→body[0]{fid(2)}
    assert!(matches!(&c1.nodes[1], FcNode::Store { effect_state, addr, value, .. }
        if *effect_state == fid(1) && *addr == fid(0) && *value == fid(2)
    ));
    // Return: own effect_args=[Store{body[1]=fid(3)}], inherited Load pruned
    assert!(matches!(&c1.terminator, FcTerm::Return { effect_args, .. }
        if effect_args.len() == 1 && effect_args[0].1 == fid(3)
    ));
}
