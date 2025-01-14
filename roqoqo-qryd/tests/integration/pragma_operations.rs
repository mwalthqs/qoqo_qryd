// Copyright © 2021-2022 HQS Quantum Simulations GmbH. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

use bincode::serialize;
use qoqo_calculator::Calculator;
use roqoqo::operations::{InvolveQubits, InvolvedQubits, Operate, PragmaChangeDevice, Substitute};
use roqoqo_qryd::pragma_operations::{PragmaChangeQRydLayout, PragmaShiftQRydQubit};
use serde_test::{assert_tokens, Configure, Token};
use std::collections::HashMap;

/// Test PragmaChangeQRydLayout inputs and involved qubits
#[test]
fn pragma_change_qryd_layout_inputs_qubits() {
    let pragma = PragmaChangeQRydLayout::new(1);

    // Test inputs are correct
    assert_eq!(pragma.new_layout(), &1_usize);

    // Test InvolveQubits trait
    assert_eq!(pragma.involved_qubits(), InvolvedQubits::All);
}

/// Test PragmaChangeQRydLayout to_pragma_change_device function
#[test]
fn pragma_change_qryd_layout_change() {
    let pragma = PragmaChangeQRydLayout::new(1);

    // Test inputs are correct
    let result = PragmaChangeDevice {
        wrapped_tags: vec![
            "Operation".to_string(),
            "PragmaOperation".to_string(),
            "PragmaChangeQRydLayout".to_string(),
        ],
        wrapped_hqslang: "PragmaChangeQRydLayout".to_string(),
        wrapped_operation: serialize(&pragma).unwrap(),
    };
    assert_eq!(pragma.to_pragma_change_device().unwrap(), result);
}

/// Test PragmaChangeQRydLayout standard derived traits (Debug, Clone, PartialEq)
#[test]
fn pragma_change_qryd_layout_simple_traits() {
    let pragma = PragmaChangeQRydLayout::new(1);
    // Test Debug trait
    assert_eq!(
        format!("{:?}", pragma),
        "PragmaChangeQRydLayout { new_layout: 1 }"
    );

    // Test Clone trait
    assert_eq!(pragma.clone(), pragma);

    // Test PartialEq trait
    let pragma_0 = PragmaChangeQRydLayout::new(1);
    let pragma_1 = PragmaChangeQRydLayout::new(2);
    assert!(pragma_0 == pragma);
    assert!(pragma == pragma_0);
    assert!(pragma_1 != pragma);
    assert!(pragma != pragma_1);
}

/// Test PragmaChangeQRydLayout Operate trait
#[test]
fn pragma_change_qryd_layout_operate_trait() {
    let pragma = PragmaChangeQRydLayout::new(1);

    // (1) Test tags function
    let tags: &[&str; 3] = &["Operation", "PragmaOperation", "PragmaChangeQRydLayout"];
    assert_eq!(pragma.tags(), tags);

    // (2) Test hqslang function
    assert_eq!(pragma.hqslang(), String::from("PragmaChangeQRydLayout"));

    // (3) Test is_parametrized function
    assert!(!pragma.is_parametrized());
}

/// Test PragmaChangeQRydLayout Substitute trait
#[test]
fn pragma_change_qryd_layout_substitute_trait() {
    let pragma = PragmaChangeQRydLayout::new(1);
    let pragma_test = PragmaChangeQRydLayout::new(1);
    // (1) Substitute parameters function
    let mut substitution_dict: Calculator = Calculator::new();
    substitution_dict.set_variable("ro", 0.0);
    let result = pragma_test
        .substitute_parameters(&substitution_dict)
        .unwrap();
    assert_eq!(result, pragma);

    // (2) Remap qubits function
    let mut qubit_mapping_test: HashMap<usize, usize> = HashMap::new();
    qubit_mapping_test.insert(0, 2);
    let result = pragma_test.remap_qubits(&qubit_mapping_test).unwrap();
    assert_eq!(result, pragma);
}

/// Test PragmaChangeQRydLayout Serialization and Deserialization traits (readable)
#[test]
fn pragma_change_qryd_layout_serde_readable() {
    let pragma_serialization = PragmaChangeQRydLayout::new(1);
    assert_tokens(
        &pragma_serialization.readable(),
        &[
            Token::Struct {
                name: "PragmaChangeQRydLayout",
                len: 1,
            },
            Token::Str("new_layout"),
            Token::U64(1),
            Token::StructEnd,
        ],
    );
}

/// Test PragmaChangeQRydLayout Serialization and Deserialization traits (compact)
#[test]
fn pragma_change_qryd_layout_serde_compact() {
    let pragma_serialization = PragmaChangeQRydLayout::new(1);
    assert_tokens(
        &pragma_serialization.compact(),
        &[
            Token::Struct {
                name: "PragmaChangeQRydLayout",
                len: 1,
            },
            Token::Str("new_layout"),
            Token::U64(1),
            Token::StructEnd,
        ],
    );
}

/// Test PragmaShiftQRydQubit inputs and involved qubits
#[test]
fn pragma_shift_qryd_qubit_inputs_qubits() {
    let mut new_positions: HashMap<usize, (usize, usize)> = HashMap::new();
    new_positions.insert(1, (0, 0));
    new_positions.insert(0, (0, 1));
    let pragma = PragmaShiftQRydQubit::new(new_positions.clone());

    // Test inputs are correct
    assert_eq!(pragma.new_positions(), &new_positions);

    // Test InvolveQubits trait
    assert_eq!(pragma.involved_qubits(), InvolvedQubits::All);
}

/// Test PragmaChangeQRydLayout to_pragma_change_device function
#[test]
fn pragma_shift_qryd_qubit_change() {
    let mut new_positions: HashMap<usize, (usize, usize)> = HashMap::new();
    new_positions.insert(1, (0, 0));
    new_positions.insert(0, (0, 1));
    let pragma = PragmaShiftQRydQubit::new(new_positions);

    // Test inputs are correct
    let result = PragmaChangeDevice {
        wrapped_tags: vec![
            "Operation".to_string(),
            "PragmaOperation".to_string(),
            "PragmaShiftQRydQubit".to_string(),
        ],
        wrapped_hqslang: "PragmaShiftQRydQubit".to_string(),
        wrapped_operation: serialize(&pragma).unwrap(),
    };
    assert_eq!(pragma.to_pragma_change_device().unwrap(), result);
}

/// Test PragmaShiftQRydQubit standard derived traits (Debug, Clone, PartialEq)
#[test]
fn pragma_shift_qryd_qubit_simple_traits() {
    let mut new_positions: HashMap<usize, (usize, usize)> = HashMap::new();
    new_positions.insert(1, (0, 0));
    new_positions.insert(0, (0, 1));
    let pragma = PragmaShiftQRydQubit::new(new_positions.clone());
    // Test Debug trait
    assert_eq!(
        format!("{:?}", pragma),
        format!(
            "PragmaShiftQRydQubit {{ new_positions: {:?} }}",
            new_positions.clone()
        )
    );

    // Test Clone trait
    assert_eq!(pragma.clone(), pragma);

    // Test PartialEq trait
    let pragma_0 = PragmaShiftQRydQubit::new(new_positions.clone());
    let pragma_1 = PragmaShiftQRydQubit::new(HashMap::new());
    assert!(pragma_0 == pragma);
    assert!(pragma == pragma_0);
    assert!(pragma_1 != pragma);
    assert!(pragma != pragma_1);
}

/// Test PragmaShiftQRydQubit Operate trait
#[test]
fn pragma_shift_qryd_qubit_operate_trait() {
    let mut new_positions: HashMap<usize, (usize, usize)> = HashMap::new();
    new_positions.insert(1, (0, 0));
    new_positions.insert(0, (0, 1));
    let pragma = PragmaShiftQRydQubit::new(new_positions.clone());

    // (1) Test tags function
    let tags: &[&str; 3] = &["Operation", "PragmaOperation", "PragmaShiftQRydQubit"];
    assert_eq!(pragma.tags(), tags);

    // (2) Test hqslang function
    assert_eq!(pragma.hqslang(), String::from("PragmaShiftQRydQubit"));

    // (3) Test is_parametrized function
    assert!(!pragma.is_parametrized());
}

/// Test PragmaShiftQRydQubit Substitute trait
#[test]
fn pragma_shift_qryd_qubit_substitute_trait() {
    let mut new_positions: HashMap<usize, (usize, usize)> = HashMap::new();
    new_positions.insert(1, (0, 0));
    new_positions.insert(0, (0, 1));
    let pragma = PragmaShiftQRydQubit::new(new_positions.clone());
    let pragma_test = PragmaShiftQRydQubit::new(new_positions.clone());
    // (1) Substitute parameters function
    let mut substitution_dict: Calculator = Calculator::new();
    substitution_dict.set_variable("ro", 0.0);
    let result = pragma_test
        .substitute_parameters(&substitution_dict)
        .unwrap();
    assert_eq!(result, pragma);

    // (2) Remap qubits function
    let mut qubit_mapping_test: HashMap<usize, usize> = HashMap::new();
    qubit_mapping_test.insert(0, 2);
    let result = pragma_test.remap_qubits(&qubit_mapping_test).unwrap();
    assert_eq!(result, pragma);
}

/// Test PragmaShiftQRydQubit Serialization and Deserialization traits (readable)
#[test]
fn pragma_shift_qryd_qubit_serde_readable() {
    let mut new_positions: HashMap<usize, (usize, usize)> = HashMap::new();
    new_positions.insert(0, (0, 0));
    let pragma_serialization = PragmaShiftQRydQubit::new(new_positions.clone());
    assert_tokens(
        &pragma_serialization.readable(),
        &[
            Token::Struct {
                name: "PragmaShiftQRydQubit",
                len: 1,
            },
            Token::Str("new_positions"),
            Token::Map { len: Some(1) },
            Token::U64(0),
            Token::Tuple { len: 2 },
            Token::U64(0),
            Token::U64(0),
            Token::TupleEnd,
            Token::MapEnd,
            Token::StructEnd,
        ],
    );
}

/// Test PragmaShiftQRydQubit Serialization and Deserialization traits (compact)
#[test]
fn pragma_shift_qryd_qubit_serde_compact() {
    let mut new_positions: HashMap<usize, (usize, usize)> = HashMap::new();
    new_positions.insert(0, (0, 0));
    let pragma_serialization = PragmaShiftQRydQubit::new(new_positions.clone());
    assert_tokens(
        &pragma_serialization.compact(),
        &[
            Token::Struct {
                name: "PragmaShiftQRydQubit",
                len: 1,
            },
            Token::Str("new_positions"),
            Token::Map { len: Some(1) },
            Token::U64(0),
            Token::Tuple { len: 2 },
            Token::U64(0),
            Token::U64(0),
            Token::TupleEnd,
            Token::MapEnd,
            Token::StructEnd,
        ],
    );
}
