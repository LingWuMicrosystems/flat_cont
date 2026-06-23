use alloc::{string::String, vec, vec::Vec};

use crate::basicblock::{
    BasicBlock, BasicBlockGraph, ContId as BbCid, Node as BBNode, NodeId as BbNid,
    Terminator as BBTerm, Value as BbValue,
};
use crate::common::RawType;
use crate::flatcont::{
    ContId as FcCid, FlatContGraph, FlatContinuation, Node as FCNode, NodeId as FcNid,
    Terminator as FCTerm, Value as FlatContValue,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn cid(x: u32) -> BbCid {
    BbCid(x)
}
fn fcid(x: u32) -> FcCid {
    FcCid(x)
}

fn dedup(v: &mut Vec<BbNid>) {
    let mut i = 0;
    while i < v.len() {
        if v[..i].contains(&v[i]) {
            v.remove(i);
        } else {
            i += 1;
        }
    }
}

fn as_node(v: &BbValue) -> BbNid {
    v.as_node()
}

fn get_inputs(node: &BBNode) -> (Vec<BbNid>, Vec<BbNid>) {
    let (mut eff, mut com) = (Vec::new(), Vec::new());
    match node {
        BBNode::Load {
            effect_state, addr, ..
        } => {
            eff.push(as_node(effect_state));
            com.push(as_node(addr));
        }
        BBNode::Store {
            effect_state,
            addr,
            value,
            ..
        } => {
            eff.push(as_node(effect_state));
            com.extend([as_node(addr), as_node(value)]);
        }
        BBNode::AtomicCAS {
            effect_state,
            addr,
            old,
            new,
            ..
        } => {
            eff.push(as_node(effect_state));
            com.extend([as_node(addr), as_node(old), as_node(new)]);
        }
        BBNode::AtomicRMW {
            effect_state,
            addr,
            value,
            ..
        } => {
            eff.push(as_node(effect_state));
            com.extend([as_node(addr), as_node(value)]);
        }
        BBNode::GEP(_, base, indices) => {
            com.push(as_node(base));
            com.extend(indices.iter().map(as_node));
        }
        BBNode::Select(c, t, f) => {
            com.extend([as_node(c), as_node(t), as_node(f)]);
        }
        BBNode::Icmp(_, a, b) => {
            com.extend([as_node(a), as_node(b)]);
        }
        BBNode::Compute(_, operands) => {
            com.extend(operands.iter().map(as_node));
        }
        BBNode::Proj(base, _) => {
            com.push(as_node(base));
        }
        BBNode::TokenMerge(effect_states) => {
            eff.extend(effect_states.iter().map(as_node));
        }
        BBNode::Call {
            target,
            effect_args,
            args,
        } => {
            if let BbValue::Node(n) = target {
                com.push(*n);
            }
            eff.extend(effect_args.iter().map(|(_, v)| as_node(v)));
            com.extend(args.iter().map(as_node));
        }
        _ => {}
    }
    dedup(&mut eff);
    dedup(&mut com);
    (eff, com)
}

fn successor_ids(term: &BBTerm) -> Vec<BbCid> {
    match term {
        BBTerm::Jump { target, .. } => vec![*target],
        BBTerm::Branch {
            then_target,
            else_target,
            ..
        } => vec![*then_target, *else_target],
        BBTerm::Switch { targets, .. } => targets.clone(),
        BBTerm::Return { .. } => vec![],
    }
}

fn compute_pres(bbs: &[BasicBlock]) -> Vec<Vec<BbCid>> {
    let mut pres: Vec<Vec<BbCid>> = (0..bbs.len()).map(|_| Vec::new()).collect();
    for (i, bb) in bbs.iter().enumerate() {
        for t in successor_ids(&bb.terminator) {
            pres[t.0 as usize].push(cid(i as u32));
        }
    }
    pres
}

// ---------------------------------------------------------------------------
// Terminator field access (read / append / set)
// ---------------------------------------------------------------------------

fn term_eff_slice(t: &BBTerm) -> &[(String, BbValue)] {
    match t {
        BBTerm::Jump { effect_args, .. }
        | BBTerm::Branch { effect_args, .. }
        | BBTerm::Switch { effect_args, .. }
        | BBTerm::Return { effect_args, .. } => effect_args,
    }
}

fn term_com_slice(t: &BBTerm) -> &[BbValue] {
    match t {
        BBTerm::Jump { common_args, .. }
        | BBTerm::Branch { common_args, .. }
        | BBTerm::Switch { common_args, .. }
        | BBTerm::Return { common_args, .. } => common_args,
    }
}

fn eff_args_mut(t: &mut BBTerm) -> &mut Vec<(String, BbValue)> {
    match t {
        BBTerm::Jump { effect_args, .. }
        | BBTerm::Branch { effect_args, .. }
        | BBTerm::Switch { effect_args, .. }
        | BBTerm::Return { effect_args, .. } => effect_args,
    }
}

fn com_args_mut(t: &mut BBTerm) -> &mut Vec<BbValue> {
    match t {
        BBTerm::Jump { common_args, .. }
        | BBTerm::Branch { common_args, .. }
        | BBTerm::Switch { common_args, .. }
        | BBTerm::Return { common_args, .. } => common_args,
    }
}

fn push_eff(t: &mut BBTerm, nid: BbNid) {
    let v = BbValue::Node(nid);
    let ea = eff_args_mut(t);
    if !ea.iter().any(|(_, n)| *n == v) {
        ea.push((String::new(), v));
    }
}

fn push_com(t: &mut BBTerm, nid: BbNid) {
    let v = BbValue::Node(nid);
    let ca = com_args_mut(t);
    if !ca.contains(&v) {
        ca.push(v);
    }
}

fn set_eff(t: &mut BBTerm, nids: &[BbNid]) {
    *eff_args_mut(t) = nids
        .iter()
        .map(|&n| (String::new(), BbValue::Node(n)))
        .collect();
}

fn set_com(t: &mut BBTerm, nids: Vec<BbNid>) {
    *com_args_mut(t) = nids.into_iter().map(BbValue::Node).collect();
}

// ---------------------------------------------------------------------------
// Propagate helpers (Pass 2)
// ---------------------------------------------------------------------------

fn propagate_com(
    bbs: &[BasicBlock],
    pres: &[Vec<BbCid>],
    end_map: &[usize],
    bb_id: usize,
) -> Vec<BbNid> {
    let own_vals = term_com_slice(&bbs[bb_id].terminator);
    let own: Vec<BbNid> = own_vals.iter().map(as_node).collect();
    let mut req = own.clone();
    for pre in &pres[bb_id] {
        for v in term_com_slice(&bbs[pre.0 as usize].terminator) {
            let n = as_node(v);
            if !req.contains(&n) {
                req.push(n);
            }
        }
    }
    req.retain(|k| {
        if own.contains(k) {
            return true;
        }
        let e = end_map[k.0 as usize];
        e != usize::MAX && e > bb_id
    });
    req
}

fn propagate_eff(
    bbs: &[BasicBlock],
    pres: &[Vec<BbCid>],
    end_map: &[usize],
    bb_id: usize,
) -> Vec<BbNid> {
    let mut own: Vec<BbNid> = Vec::new();
    for (_, v) in term_eff_slice(&bbs[bb_id].terminator) {
        own.push(as_node(v));
    }
    let mut req = own.clone();
    for pre in &pres[bb_id] {
        for (_, v) in term_eff_slice(&bbs[pre.0 as usize].terminator) {
            let n = as_node(v);
            if !req.contains(&n) {
                req.push(n);
            }
        }
    }
    req.retain(|k| {
        if own.contains(k) {
            return true;
        }
        let e = end_map[k.0 as usize];
        e != usize::MAX && e > bb_id
    });
    req
}

// ---------------------------------------------------------------------------
// Main algorithm
// ---------------------------------------------------------------------------

pub fn bb_to_flat_cont(graph: &BasicBlockGraph) -> FlatContGraph {
    let bbs = &graph.bbs;
    let g_nodes = &graph.nodes;
    let num_bbs = bbs.len();
    let num_nodes = g_nodes.len();
    let pres = compute_pres(bbs);

    // ---- Build node-to-bb map + visible sets ----
    let (node_to_bb, visible) = bbs.iter().enumerate().fold(
        (vec![usize::MAX; num_nodes], vec![Vec::new(); num_bbs]),
        |(mut nmap, mut vis), (bb_id, bb)| {
            for n in &bb.node_ids {
                nmap[n.0 as usize] = bb_id;
            }
            let mut seen = vec![false; num_nodes];
            for n in &bb.node_ids {
                if !seen[n.0 as usize] {
                    seen[n.0 as usize] = true;
                    vis[bb_id].push(*n);
                }
            }
            for pre in &pres[bb_id] {
                let pre_vis: Vec<BbNid> = vis[pre.0 as usize].to_vec();
                for n in pre_vis {
                    if !seen[n.0 as usize] {
                        seen[n.0 as usize] = true;
                        vis[bb_id].push(n);
                    }
                }
            }
            (nmap, vis)
        },
    );

    // ---- Pass 1: discover cross-BB deps, push to predecessor terminators ----
    let (bbs_mut, end_eff, end_com) = (0..num_bbs).fold(
        (
            bbs.clone(),
            vec![usize::MAX; num_nodes],
            vec![usize::MAX; num_nodes],
        ),
        |(mut acc, mut eff_end, mut com_end), bb_id| {
            let node_ids = acc[bb_id].node_ids.clone();
            for nid in &node_ids {
                let (effs, coms) = get_inputs(&g_nodes[nid.0 as usize]);

                for inp in &effs {
                    let src = node_to_bb[inp.0 as usize];
                    if src == bb_id {
                        continue;
                    }
                    for pre in &pres[bb_id] {
                        if visible[pre.0 as usize].contains(inp) {
                            push_eff(&mut acc[pre.0 as usize].terminator, *inp);
                        }
                        eff_end[inp.0 as usize] = bb_id;
                    }
                }
                for inp in &coms {
                    let src = node_to_bb[inp.0 as usize];
                    if src == bb_id {
                        continue;
                    }
                    for pre in &pres[bb_id] {
                        if visible[pre.0 as usize].contains(inp) {
                            push_com(&mut acc[pre.0 as usize].terminator, *inp);
                        }
                        com_end[inp.0 as usize] = bb_id;
                    }
                }
            }
            (acc, eff_end, com_end)
        },
    );

    // ---- Pass 2: propagate + prune ----
    let bbs_mut = (0..num_bbs).fold(bbs_mut, |mut acc, bb_id| {
        let eff = propagate_eff(&acc, &pres, &end_eff, bb_id);
        let com = propagate_com(&acc, &pres, &end_com, bb_id);
        set_eff(&mut acc[bb_id].terminator, &eff);
        set_com(&mut acc[bb_id].terminator, com);
        acc
    });

    // ---- Remap to FlatContinuation ----
    let continuations = remap_conts(&bbs_mut, g_nodes);

    FlatContGraph {
        name: graph.name.clone(),
        static_data: graph.static_data.clone(),
        externals: graph.externals.clone(),
        continuations,
    }
}

// ---------------------------------------------------------------------------
// Remap global NodeIds → local FlatContinuation indices
// ---------------------------------------------------------------------------

fn remap_conts(bbs: &[BasicBlock], g_nodes: &[BBNode]) -> Vec<FlatContinuation> {
    let num_nodes = g_nodes.len();
    bbs.iter()
        .map(|bb| {
            let mut p_ids: Vec<BbNid> = Vec::new();
            let mut e_ids: Vec<BbNid> = Vec::new();
            let mut seen = vec![false; num_nodes];

            // External refs from node inputs
            for nid in &bb.node_ids {
                let (effs, coms) = get_inputs(&g_nodes[nid.0 as usize]);
                for id in &coms {
                    if !bb.node_ids.contains(id) && !seen[id.0 as usize] {
                        seen[id.0 as usize] = true;
                        p_ids.push(*id);
                    }
                }
                for id in &effs {
                    if !bb.node_ids.contains(id) && !seen[id.0 as usize] {
                        seen[id.0 as usize] = true;
                        e_ids.push(*id);
                    }
                }
            }

            // External refs from terminator args
            for v in term_com_slice(&bb.terminator) {
                let n = as_node(v);
                if !bb.node_ids.contains(&n) && !seen[n.0 as usize] {
                    seen[n.0 as usize] = true;
                    p_ids.push(n);
                }
            }
            for (_, v) in term_eff_slice(&bb.terminator) {
                let n = as_node(v);
                if !bb.node_ids.contains(&n) && !seen[n.0 as usize] {
                    seen[n.0 as usize] = true;
                    e_ids.push(n);
                }
            }

            // Own Param / EffectParam in node_ids
            for nid in &bb.node_ids {
                if seen[nid.0 as usize] {
                    continue;
                }
                seen[nid.0 as usize] = true;
                match &g_nodes[nid.0 as usize] {
                    BBNode::Param(..) => p_ids.push(*nid),
                    BBNode::EffectParam(..) => e_ids.push(*nid),
                    _ => {}
                }
            }

            // Local index mapping
            let body_off = p_ids.len() + e_ids.len();
            let mut loc = vec![usize::MAX; num_nodes];
            for (i, id) in p_ids.iter().enumerate() {
                loc[id.0 as usize] = i;
            }
            for (i, id) in e_ids.iter().enumerate() {
                loc[id.0 as usize] = p_ids.len() + i;
            }

            // Body nodes (skip Param / EffectParam)
            let mut body: Vec<FCNode> = Vec::new();
            for nid in &bb.node_ids {
                if matches!(
                    &g_nodes[nid.0 as usize],
                    BBNode::Param(..) | BBNode::EffectParam(..)
                ) {
                    continue;
                }
                loc[nid.0 as usize] = body_off + body.len();
                body.push(remap_node(&g_nodes[nid.0 as usize], &loc));
            }

            let params: Vec<RawType> = p_ids
                .iter()
                .map(|id| infer_type(&g_nodes[id.0 as usize]))
                .collect();
            let effects: Vec<String> = e_ids
                .iter()
                .map(|id| effect_name(&g_nodes[id.0 as usize]))
                .collect();
            let term = remap_term(&bb.terminator, &loc);

            FlatContinuation {
                effects,
                params,
                nodes: body,
                terminator: term,
            }
        })
        .collect()
}

fn infer_type(node: &BBNode) -> RawType {
    match node {
        BBNode::Const(_, t)
        | BBNode::Load { data_type: t, .. }
        | BBNode::Store { data_type: t, .. }
        | BBNode::AtomicCAS { data_type: t, .. }
        | BBNode::AtomicRMW { data_type: t, .. }
        | BBNode::GEP(t, ..) => t.clone(),
        BBNode::Icmp(..) | BBNode::Select(..) => RawType::Scalar(1),
        BBNode::Compute(..) => RawType::Scalar(32),
        BBNode::Proj(..) => RawType::Token,
        BBNode::TokenMerge(..) => RawType::Token,
        BBNode::Param(..) | BBNode::EffectParam(..) => RawType::Token,
        _ => RawType::Token,
    }
}

fn effect_name(_node: &BBNode) -> String {
    String::new()
}

fn remap_val(v: &BbValue, m: &[usize]) -> FlatContValue {
    match v {
        BbValue::Node(n) => FlatContValue::Node(FcNid(m[n.0 as usize] as u32)),
        BbValue::Const(val, t) => FlatContValue::Const(*val, t.clone()),
        BbValue::Param(i) => FlatContValue::Param(*i),
        BbValue::Effect(i) => FlatContValue::Effect(*i),
    }
}

fn remap_node(node: &BBNode, m: &[usize]) -> FCNode {
    match node {
        BBNode::Const(v, t) => FCNode::Const(*v, t.clone()),
        BBNode::DataRef(d) => FCNode::DataRef(*d),
        BBNode::ExternRef(e) => FCNode::ExternRef(*e),
        BBNode::ContRef(c) => FCNode::ContRef(fcid(c.0)),
        BBNode::Load {
            data_type,
            effect_state,
            addr,
            signed,
        } => FCNode::Load {
            data_type: data_type.clone(),
            effect_state: remap_val(effect_state, m),
            addr: remap_val(addr, m),
            signed: *signed,
        },
        BBNode::Store {
            data_type,
            effect_state,
            addr,
            value,
        } => FCNode::Store {
            data_type: data_type.clone(),
            effect_state: remap_val(effect_state, m),
            addr: remap_val(addr, m),
            value: remap_val(value, m),
        },
        BBNode::AtomicCAS {
            data_type,
            effect_state,
            addr,
            old,
            new,
        } => FCNode::AtomicCAS {
            data_type: data_type.clone(),
            effect_state: remap_val(effect_state, m),
            addr: remap_val(addr, m),
            old: remap_val(old, m),
            new: remap_val(new, m),
        },
        BBNode::AtomicRMW {
            data_type,
            effect_state,
            addr,
            value,
            operator,
        } => FCNode::AtomicRMW {
            data_type: data_type.clone(),
            effect_state: remap_val(effect_state, m),
            addr: remap_val(addr, m),
            value: remap_val(value, m),
            operator: operator.clone(),
        },
        BBNode::GEP(t, base, indices) => FCNode::GEP(
            t.clone(),
            remap_val(base, m),
            indices.iter().map(|v| remap_val(v, m)).collect(),
        ),
        BBNode::Select(c, t, f) => {
            FCNode::Select(remap_val(c, m), remap_val(t, m), remap_val(f, m))
        }
        BBNode::Icmp(cond, a, b) => FCNode::Icmp(cond.clone(), remap_val(a, m), remap_val(b, m)),
        BBNode::Compute(op, ops) => {
            FCNode::Compute(op.clone(), ops.iter().map(|v| remap_val(v, m)).collect())
        }
        BBNode::Proj(base, idx) => FCNode::Proj(remap_val(base, m), *idx),
        BBNode::TokenMerge(effect_states) => {
            FCNode::TokenMerge(effect_states.iter().map(|v| remap_val(v, m)).collect())
        }
        BBNode::Call {
            target,
            effect_args,
            args,
        } => FCNode::Call {
            target: remap_val(target, m),
            effect_args: effect_args
                .iter()
                .map(|(s, v)| (s.clone(), remap_val(v, m)))
                .collect(),
            args: args.iter().map(|v| remap_val(v, m)).collect(),
        },
        BBNode::Param(..) | BBNode::EffectParam(..) => panic!("param in body"),
    }
}

fn remap_term(term: &BBTerm, m: &[usize]) -> FCTerm {
    match term {
        BBTerm::Jump {
            effect_args,
            common_args,
            target,
        } => FCTerm::Jump {
            effect_args: effect_args
                .iter()
                .map(|(s, v)| (s.clone(), remap_val(v, m)))
                .collect(),
            common_args: common_args.iter().map(|v| remap_val(v, m)).collect(),
            target: fcid(target.0),
        },
        BBTerm::Branch {
            effect_args,
            common_args,
            cond,
            then_target,
            else_target,
        } => FCTerm::Branch {
            effect_args: effect_args
                .iter()
                .map(|(s, v)| (s.clone(), remap_val(v, m)))
                .collect(),
            common_args: common_args.iter().map(|v| remap_val(v, m)).collect(),
            cond: remap_val(cond, m),
            then_target: fcid(then_target.0),
            else_target: fcid(else_target.0),
        },
        BBTerm::Switch {
            effect_args,
            common_args,
            case,
            targets,
        } => FCTerm::Switch {
            effect_args: effect_args
                .iter()
                .map(|(s, v)| (s.clone(), remap_val(v, m)))
                .collect(),
            common_args: common_args.iter().map(|v| remap_val(v, m)).collect(),
            case: remap_val(case, m),
            targets: targets.iter().map(|t| fcid(t.0)).collect(),
        },
        BBTerm::Return {
            effect_args,
            common_args,
        } => FCTerm::Return {
            effect_args: effect_args
                .iter()
                .map(|(s, v)| (s.clone(), remap_val(v, m)))
                .collect(),
            common_args: common_args.iter().map(|v| remap_val(v, m)).collect(),
        },
    }
}
