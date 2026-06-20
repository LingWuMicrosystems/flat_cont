use flat_cont::basicblock::{
    BasicBlock, BasicBlockGraph, Node as BbNode, NodeId as BbNid, Terminator as BbTerm,
    Value as BbValue,
};
use flat_cont::bb2flatcont::bb_to_flat_cont;
use flat_cont::common::RawType;
use flat_cont::flatcont::{
    Node as FcNode, NodeId as FcNid, Terminator as FcTerm, Value as FcValue,
};

fn fid(x: u32) -> FcNid {
    FcNid(x)
}
fn cid(x: u32) -> flat_cont::basicblock::ContId {
    flat_cont::basicblock::ContId(x)
}
fn bb_val(x: u32) -> BbValue {
    BbValue::Node(BbNid(x))
}
fn fc_val(x: u32) -> FcValue {
    FcValue::Node(FcNid(x))
}

// BB0: EffectParam → Load(effect_param) → Proj(Load, 1)  ─→  BB1
// BB1: Store(effect=Proj(Load)) → Proj(Store, 0)  ─→  Return(effect=Proj(Store))
fn make_effect_chain_graph() -> BasicBlockGraph {
    BasicBlockGraph {
        name: Some("effect_chain".into()),
        static_data: vec![],
        externals: vec![],
        nodes: vec![
            BbNode::EffectParam(0),          // 0: initial effect
            BbNode::Const(16, RawType::Ptr), // 1: addr
            BbNode::Load {
                data_type: RawType::Scalar(32),
                effect_state: bb_val(0),
                addr: bb_val(1),
                signed: false,
            }, // 2: load
            BbNode::Proj(bb_val(2), 1),         // 3: new effect from Load
            BbNode::Const(42, RawType::Scalar(32)), // 4: value
            BbNode::Store {
                data_type: RawType::Scalar(32),
                effect_state: bb_val(3),
                addr: bb_val(1),
                value: bb_val(4),
            }, // 5: store
            BbNode::Proj(bb_val(5), 0),         // 6: new effect from Store
        ],
        bbs: vec![
            BasicBlock {
                node_ids: vec![BbNid(0), BbNid(1), BbNid(2), BbNid(3)],
                terminator: BbTerm::Jump {
                    effect_args: vec![],
                    common_args: vec![],
                    target: cid(1),
                },
            },
            BasicBlock {
                node_ids: vec![BbNid(4), BbNid(5), BbNid(6)],
                terminator: BbTerm::Return {
                    effect_args: vec![(String::new(), bb_val(6))],
                    common_args: vec![],
                },
            },
        ],
    }
}

#[test]
fn test_effect_chain() {
    let graph = make_effect_chain_graph();
    let result = bb_to_flat_cont(&graph);

    assert_eq!(result.continuations.len(), 2);

    // ---- Cont 0: effects=[EffectParam], body=[Const{1}, Load{2}, Proj_load_eff{3}] ----
    let c0 = &result.continuations[0];
    assert_eq!(c0.effects.len(), 1); // EffectParam
    assert_eq!(c0.params.len(), 0);
    assert_eq!(c0.nodes.len(), 3);
    // Load at body[1]{fid=2}: effect_state→effects[0], addr→body[0]
    assert!(
        matches!(&c0.nodes[1], FcNode::Load { effect_state, addr, .. }
            if *effect_state == fc_val(0) && *addr == fc_val(1)
        )
    );
    // Proj at body[2]{fid=3}: base→Load{body[1]=fid(2)}
    assert!(matches!(&c0.nodes[2], FcNode::Proj(base, 1) if *base == fc_val(2)));
    // Jump: effect_args=[Proj_load_eff{body[2]=fid(3)}]
    assert!(matches!(&c0.terminator, FcTerm::Jump { effect_args, .. }
        if effect_args.len() == 1 && effect_args[0].1 == fc_val(3)
    ));

    // ---- Cont 1: effects=[Proj_load], params=[addr], body=[Const{4}, Store{5}, Proj_store_eff{6}] ----
    let c1 = &result.continuations[1];
    assert_eq!(c1.effects.len(), 1); // Proj(Load,1)
    assert_eq!(c1.params.len(), 1); // addr(1) is external → param
    assert_eq!(c1.nodes.len(), 3);
    //  P=1, E=1, body_offset=2. params[0]=fid(0), effects[0]=fid(1), body=[fid(2), fid(3), fid(4)]
    // Store at body[1]{fid=3}: effect_state→effects[0]{fid(1)}, addr→params[0]{fid(0)}, value→body[0]{fid(2)}
    assert!(
        matches!(&c1.nodes[1], FcNode::Store { effect_state, addr, value, .. }
            if *effect_state == fc_val(1) && *addr == fc_val(0) && *value == fc_val(2)
        )
    );
    // Proj at body[2]{fid=4}: base→Store{body[1]=fid(3)}
    assert!(matches!(&c1.nodes[2], FcNode::Proj(base, 0) if *base == fc_val(3)));
    // Return: effect_args=[Proj_store_eff{body[2]=fid(4)}]
    assert!(matches!(&c1.terminator, FcTerm::Return { effect_args, .. }
        if effect_args.len() == 1 && effect_args[0].1 == fc_val(4)
    ));
}
