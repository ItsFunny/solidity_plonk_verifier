use std::io::Write;
use bellman_ce::pairing::ff::{PrimeField, PrimeFieldRepr};
use bellman_ce::pairing::{Engine, CurveAffine};
use bellman_ce::plonk::better_cs::keys::{VerificationKey, Proof};
use bellman_ce::pairing::bn256::{Bn256, Fr};
use bellman_ce::plonk::better_cs::cs::PlonkCsWidth4WithNextStepParams;
use bellman_ce::plonk::domains::Domain;
use handlebars::*;

use serde_json::value::{Map};

use web3::types::U256;
use crate::{render_g1_affine_to_hex, render_g2_affine_to_hex, render_scalar_to_hex};

pub fn render_verification_key_from_default_template_with_writer<W: Write>(vk: &VerificationKey<Bn256, PlonkCsWidth4WithNextStepParams>,mut writer: W) {
    let template = include_str!("../template.sol");
    render_verification_key_with_writer(vk, template, writer);
}


pub fn render_verification_key_with_writer<W: Write>(vk: &VerificationKey<Bn256, PlonkCsWidth4WithNextStepParams>,
                                                     template:&str,
                                                     mut writer: W) {
    let mut map = Map::new();

    let domain_size = vk.n.next_power_of_two().to_string();
    map.insert("domain_size".to_owned(), to_json(domain_size));

    let num_inputs = vk.num_inputs.to_string();
    map.insert("num_inputs".to_owned(), to_json(num_inputs));

    let domain = bellman_ce::plonk::domains::Domain::<Fr>::new_for_size(vk.n.next_power_of_two() as u64).unwrap();
    let omega = domain.generator;
    map.insert("omega".to_owned(), to_json(render_scalar_to_hex(&omega)));

    for (i, c) in vk.selector_commitments.iter().enumerate() {
        let rendered = render_g1_affine_to_hex::<Bn256>(&c);

        for j in 0..2 {
            map.insert(format!("selector_commitment_{}_{}", i, j), to_json(&rendered[j]));
        }
    }

    for (i, c) in vk.next_step_selector_commitments.iter().enumerate() {
        let rendered = render_g1_affine_to_hex::<Bn256>(&c);

        for j in 0..2 {
            map.insert(format!("next_step_selector_commitment_{}_{}", i, j), to_json(&rendered[j]));
        }
    }

    for (i, c) in vk.permutation_commitments.iter().enumerate() {
        let rendered = render_g1_affine_to_hex::<Bn256>(&c);

        for j in 0..2 {
            map.insert(format!("permutation_commitment_{}_{}", i, j), to_json(&rendered[j]));
        }
    }

    for (i, c) in vk.non_residues.iter().enumerate() {
        let rendered = render_scalar_to_hex::<Fr>(&c);

        map.insert(format!("permutation_non_residue_{}", i), to_json(&rendered));
    }

    let rendered = render_g2_affine_to_hex(&vk.g2_elements[1]);

    map.insert("g2_x_x_c0".to_owned(), to_json(&rendered[0]));
    map.insert("g2_x_x_c1".to_owned(), to_json(&rendered[1]));
    map.insert("g2_x_y_c0".to_owned(), to_json(&rendered[2]));
    map.insert("g2_x_y_c1".to_owned(), to_json(&rendered[3]));

    let mut handlebars = Handlebars::new();

    // register template from a file and assign a name to it
    handlebars.register_template_file("contract", template).expect("must read the template");

    // make data and render it
    // println!("{}", handlebars.render("contract", &map).unwrap());


    let rendered = handlebars.render("contract", &map).unwrap();

    writer.write(rendered.as_bytes()).expect("must write to successfully");
}

#[test]
fn render_key_with_writer() {
    let mut reader = std::io::BufReader::with_capacity(1 << 24,
                                                       std::fs::File::open("/Users/lvcong/rust/solidity_plonk_verifier/bellman_vk_codegen/deposit_vk.key").unwrap(),
    );
    let vk = VerificationKey::<Bn256, PlonkCsWidth4WithNextStepParams>::read(&mut reader).unwrap();
    let mut vec_writer = Vec::<u8>::new();
    render_verification_key_with_writer(&vk, "",&mut vec_writer);
    println!("{:?}", String::from_utf8(vec_writer).unwrap().to_string());
}