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

use crate::api_devices::QRydAPIDevice;
use bitvec::prelude::*;
use num_complex::Complex64;
use roqoqo::backends::RegisterResult;
use roqoqo::measurements::ClassicalRegister;
use roqoqo::operations::Define;
use roqoqo::operations::Operation;
use roqoqo::prelude::EvaluatingBackend;
use roqoqo::Circuit;
use roqoqo::QuantumProgram;
use roqoqo::RoqoqoBackendError;
use std::collections::HashMap;
use std::env;
use std::{thread, time};

/// QRyd WebAPI backend.
///
/// The WebAPI backend implements methods available in the QRyd Web API.
/// Furthermore, QRyd quantum computer only allows gate operations
/// that are available on a device model of a QRyd device (stored in a [crate::QRydDevice]).
/// This limitation is introduced by design to check the compatability of quantum programs with a model of the QRyd hardware.
/// For simulations of the QRyd quantum computer use the backend simulator [crate::Backend].
///
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct APIBackend {
    /// Device representing the model of a QRyd device.
    pub device: QRydAPIDevice,
    /// Access token for identification with QRyd devices
    access_token: String,
    /// Timeout for synchronous EvaluatingBackend trait. In the evaluating trait.
    /// In synchronous operation the WebAPI is queried every 30 seconds until it has
    /// been queried `timeout` times.
    timeout: usize,
}

/// Local struct representing the body of the request message
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct QRydRunData {
    // The QRyd WebAPI Backend used to execute operations and circuits.
    // At the moment limited to the QRyd emulators
    // ('qryd_emu_localcomp_square', 'qryd_emu_localcomp_triangle',
    // 'qryd_emu_cloudcomp_square', 'qryd_emu_cloudcomp_triangle')
    backend: String,
    //
    develop: bool,
    // Seed
    seed: usize,
    // Phase angle for the basis gate 'PhaseShiftedControllZ'.
    pcz_theta: f64,
    // Roqoqo QuantumProgram to be executed.
    program: QuantumProgram,
}

/// Local struct representing the body of a validation error message
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ValidationError {
    detail: ValidationErrorDetail,
}

/// Local struct representing the body of a validation error message
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ValidationErrorDetail {
    #[serde(default)]
    loc: Vec<String>,
    #[serde(default)]
    msg: String,
    #[serde(alias = "type")]
    #[serde(default)]
    internal_type: String,
}

// Struct to represent QRyd response message
//
// Code |   Description
//------------------------------------------
// 201 | Successful Response
// 422 | Validation Error
//
#[derive(serde::Serialize, serde::Deserialize)]
struct QRydRunResponse {
    // the json body "detail" includes String fields "loc", "msg", "type"
    #[serde(default)] //?
    detail: String,
} // TBD: When/where is the job id communicated back to the client?

// Struct used when calling QRyd WebAPI upon a given job id.
#[derive(serde::Serialize, serde::Deserialize)]
struct QRydJobQuerry {
    // job id retrieved from QRyd WebAPI
    id: String,
    // Access token for identification with QRyd devices
    access_token: String,
}

/// Struct to represent QRyd response when calling for the Job status.
#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct QRydJobStatus {
    /// status of the job, e.g. "pending"
    #[serde(default)] // for optional fields
    pub status: String,
    /// message, if any
    #[serde(default)]
    pub msg: String,
}

/// Struct to represent QRyd response on the result for the posted Job.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
pub struct QRydJobResult {
    /// The actual measured data
    #[serde(default)]
    pub data: ResultCounts,
    /// Time taken to run and return the result
    #[serde(default)]
    pub time_taken: f64,
    #[serde(default)]
    /// The noise that was used in the run
    pub noise: String,
    #[serde(default)]
    /// The method that was used for the run
    pub method: String,
    #[serde(default)]
    /// The device that was used for the run
    pub device: String,
    // #[serde(default)]
    // /// The used precision
    // precision: String,
    #[serde(default)]
    /// The number of qubits that were used in the run
    pub num_qubits: u32,
    /// Number of classical bits
    #[serde(default)]
    pub num_clbits: u32,
    #[serde(default)]
    /// Max qubits
    pub fusion_max_qubits: u32,
    #[serde(default)]
    /// Average qubits
    pub fusion_avg_qubits: f64,
    #[serde(default)]
    /// Number of gates generated by gate fusion
    pub fusion_generated_gates: u32,
    #[serde(default)]
    /// Number of single qubit gates actually executed in the circuit
    pub executed_single_qubit_gates: u32,
    #[serde(default)]
    /// Number of two qubit gates actually executed in the circuit
    pub executed_two_qubit_gates: u32,
}

/// Represents the counts of measurements returned by QRyd API
///
/// Format corresponds to qiskit count format e.g.
///
/// ```python
/// counts = {'0x1': 100.0, '0x4': 20.0}
/// ```
///
/// where out of a total of 120 measurements 100 times
/// qubit 0 was measured in state |1> while the same measurement gave |0> for
/// qubits 1 and 2 and 20 times qubit 2 was measured in state |1>
/// with qubits 1 and 0 in state |0>
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct ResultCounts {
    /// The dictionary of counts for each measured string
    pub counts: HashMap<String, u64>,
}

impl APIBackend {
    /// Creates a new QRyd WebAPI backend.
    ///
    /// # Arguments
    ///
    /// * `device` - The QRyd device the Backend uses to execute operations and circuits.
    ///                     At the moment limited to the QRyd emulator.
    /// * `access_token` - An access_token is required to access QRYD hardware and emulators.
    ///                                 The access_token can either be given as an argument here
    ///                                 or set via the environmental variable `$QRYD_API_TOKEN`
    /// * `timeout` - Timeout for synchronous EvaluatingBackend trait. In the evaluating trait.
    ///               In synchronous operation the WebAPI is queried every 30 seconds until it has
    ///               been queried `timeout` times.
    pub fn new(
        device: QRydAPIDevice,
        access_token: Option<String>,
        timeout: Option<usize>,
    ) -> Result<Self, RoqoqoBackendError> {
        let access_token_internal: String = match access_token {
            Some(s) => s,
            None => env::var("QRYD_API_TOKEN").map_err(|_| {
                RoqoqoBackendError::MissingAuthentification {
                    msg: "QRYD access token is missing".to_string(),
                }
            })?,
        };

        Ok(Self {
            device,
            access_token: access_token_internal,
            timeout: timeout.unwrap_or(30),
        })
    }

    /// Post to add a new job to be run on the backend and return the location of the job.
    ///
    /// Other free parameters of the job (`seed`, `pcz_theta` etc.)
    /// are provided by the device given during the initializing of the backend.
    ///
    /// The returned location is the URL of the job in String form
    /// that can be used to query the job status and result
    /// or to delete the job.
    ///
    /// # Arguments
    ///
    /// * `quantumprogram` - Roqoqo QuantumProgram to be executed.
    ///
    pub fn post_job(&self, quantumprogram: QuantumProgram) -> Result<String, RoqoqoBackendError> {
        // Prepare data that need to be passed to the WebAPI client
        let seed_param: usize = self.device.seed(); // seed.unwrap_or(0);
        let theta_param: f64 = self.device.pcz_theta(); // pcz_theta.unwrap_or(0.0);
        match &quantumprogram {
            QuantumProgram::ClassicalRegister { measurement, .. } => {
                if measurement.circuits.len() != 1 {
                    return Err(RoqoqoBackendError::GenericError { msg: "QRyd API Backend only supports posting ClassicalRegister with one circuit".to_string() });
                }
                if measurement.circuits[0].is_parametrized() {
                    return Err(RoqoqoBackendError::GenericError { msg: "Qoqo circuit contains symbolic parameters. The QrydWebAPI does not support symbolic parameters.".to_string() });
                }
            }
            _ => {
                return Err(RoqoqoBackendError::GenericError {
                    msg: "QRyd API Backend only supports posting ClassicalRegister QuantumPrograms"
                        .to_string(),
                })
            }
        }
        let data = QRydRunData {
            backend: self.device.qrydbackend(),
            seed: seed_param,
            develop: false,
            pcz_theta: theta_param,
            program: quantumprogram,
        };

        // Prepare WebAPI client
        let client = reqwest::blocking::Client::builder()
            .https_only(true)
            .build()
            .map_err(|x| RoqoqoBackendError::NetworkError {
                msg: format!("could not create https client {:?}", x),
            })?;

        // Call WebAPI client
        // here: value for put() temporarily fixed.
        // needs to be derived dynamically based on the provided parameter 'qrydbackend'
        let resp = client
            .post("https://api.qryddemo.itp3.uni-stuttgart.de/v2_0/jobs")
            .header("X-API-KEY", self.access_token.clone())
            .json(&data)
            .send()
            .map_err(|e| RoqoqoBackendError::NetworkError {
                msg: format!("{:?}", e),
            })?;
        let status_code = resp.status();
        if status_code != reqwest::StatusCode::CREATED {
            if status_code == reqwest::StatusCode::UNPROCESSABLE_ENTITY {
                let querry_response: ValidationError =
                    resp.json::<ValidationError>().map_err(|e| {
                        RoqoqoBackendError::NetworkError {
                            msg: format!("Error parsing ValidationError message {:?}", e),
                        }
                    })?;
                return Err(RoqoqoBackendError::GenericError{msg:
                    format!( "QuantumProgram or metadata could not be parsed by QRyd Web-API Backend. msg: {} type: {}, loc: {:?}",querry_response.detail.msg, querry_response.detail.internal_type, querry_response.detail.loc,  )
            });
            }
            return Err(RoqoqoBackendError::NetworkError {
                msg: format!(
                    "Request to server failed with HTTP status code {:?}",
                    status_code
                ),
            });
        } else {
            let resp_headers = resp.headers();
            if resp_headers.contains_key("Location") {
                Ok(resp_headers["Location"]
                    .to_str()
                    .map_err(|err| RoqoqoBackendError::NetworkError {
                        msg: format!("Server response missing the Location header {:?}", err),
                    })?
                    .to_string())
            } else {
                Err(RoqoqoBackendError::NetworkError {
                    msg: "Server response missing the Location header".to_string(),
                })
            }
        }
    }

    /// Get status of a posted WebAPI job.
    ///
    /// # Arguments
    ///
    /// * `job_location` - location (url) of the job one is interested in.
    ///
    /// # Returns
    ///
    /// * QRydJobStatus - status and message of the job.
    /// * RoqoqoBackendError in case of a network failure.
    ///
    pub fn get_job_status(
        &self,
        job_location: String,
    ) -> Result<QRydJobStatus, RoqoqoBackendError> {
        // Prepare message body
        // let data = QRydJobQuerry {
        //     id: job_id,
        //     access_token: self.access_token.clone(),
        // };
        // Prepare WebAPI client
        let client = reqwest::blocking::Client::builder()
            .https_only(true)
            .build()
            .map_err(|x| RoqoqoBackendError::NetworkError {
                msg: format!("could not create https client {:?}", x),
            })?;

        let url_string: String = job_location + "/status";

        // Call WebAPI client
        let resp = client
            .get(url_string)
            .header("X-API-KEY", self.access_token.clone())
            // .json(&data)
            .send()
            .map_err(|e| RoqoqoBackendError::NetworkError {
                msg: format!("{:?}", e),
            })?;

        let status_code = resp.status();
        if status_code != reqwest::StatusCode::OK {
            if status_code == reqwest::StatusCode::UNPROCESSABLE_ENTITY {
                let querry_response: ValidationError =
                    resp.json::<ValidationError>().map_err(|e| {
                        RoqoqoBackendError::NetworkError {
                            msg: format!("Error parsing ValidationError message {:?}", e),
                        }
                    })?;
                return Err(RoqoqoBackendError::GenericError{msg:
                    format!( "QuantumProgram or metadata could not be parsed by QRyd Web-API Backend. msg: {} type: {}, loc: {:?}",querry_response.detail.msg, querry_response.detail.internal_type, querry_response.detail.loc,  )
            });
            }
            return Err(RoqoqoBackendError::NetworkError {
                msg: format!(
                    "Request to server failed with HTTP status code {:?}",
                    status_code
                ),
            });
        } else {
            // response object includes the fields `status` and `msg` that can be accessed if required
            let response: Result<QRydJobStatus, RoqoqoBackendError> = resp
                .json::<QRydJobStatus>()
                .map_err(|e| RoqoqoBackendError::NetworkError {
                    msg: format!("second {:?}", e),
                });
            response
        }
    }

    /// Get status of a completed WebAPI job.
    ///
    /// # Arguments
    ///
    /// * `job_location` - location (url) of the job one is interested in.
    ///
    /// # Returns
    /// * Result of the job.
    /// * RoqoqoBackendError in case of a network failure.
    ///
    pub fn get_job_result(
        &self,
        job_location: String,
    ) -> Result<QRydJobResult, RoqoqoBackendError> {
        // Prepare message body
        // let data = QRydJobQuerry {
        //     id: job,
        //     access_token: self.access_token.clone(),
        // };
        // Prepare WebAPI client
        let client = reqwest::blocking::Client::builder()
            .https_only(true)
            .build()
            .map_err(|x| RoqoqoBackendError::NetworkError {
                msg: format!("could not create https client {:?}", x),
            })?;

        // construct URL with {job_id} not required?
        let url_string: String = job_location + "/result";

        // Call WebAPI client
        let resp = client
            .get(url_string)
            .header("X-API-KEY", self.access_token.clone())
            // .json(&data)
            .send()
            .map_err(|e| RoqoqoBackendError::NetworkError {
                msg: format!("{:?}", e),
            })?;

        let status_code = resp.status();
        if status_code != reqwest::StatusCode::OK {
            if status_code == reqwest::StatusCode::UNPROCESSABLE_ENTITY {
                let querry_response: ValidationError =
                    resp.json::<ValidationError>().map_err(|e| {
                        RoqoqoBackendError::NetworkError {
                            msg: format!("Error parsing ValidationError message {:?}", e),
                        }
                    })?;
                return Err(RoqoqoBackendError::GenericError{msg:
                    format!( "QuantumProgram or metadata could not be parsed by QRyd Web-API Backend. msg: {} type: {}, loc: {:?}",querry_response.detail.msg, querry_response.detail.internal_type, querry_response.detail.loc,  )
            });
            }
            return Err(RoqoqoBackendError::NetworkError {
                msg: format!(
                    "Request to server failed with HTTP status code {:?}",
                    status_code
                ),
            });
        } else {
            // response object
            let response: Result<QRydJobResult, RoqoqoBackendError> = resp
                .json::<QRydJobResult>()
                .map_err(|e| RoqoqoBackendError::NetworkError {
                    msg: format!("Error parsing job status response {:?}", e),
                });
            response
        }
    }

    /// Delete a posted WebAPI job
    ///
    /// # Arguments
    ///
    /// * `job_location` - location (url) of the job one is interested in.
    ///
    /// # Returns
    /// * RoqoqoBackendError in case of a network failure.
    ///
    pub fn delete_job(&self, job_location: String) -> Result<(), RoqoqoBackendError> {
        // Prepare WebAPI client
        let client = reqwest::blocking::Client::builder()
            .https_only(true)
            .build()
            .map_err(|x| RoqoqoBackendError::NetworkError {
                msg: format!("could not create https client {:?}", x),
            })?;
        // Call WebAPI client
        let resp = client
            .delete(job_location)
            .header("X-API-KEY", self.access_token.clone())
            .send()
            .map_err(|e| RoqoqoBackendError::NetworkError {
                msg: format!("{:?}", e),
            })?;

        let status_code = resp.status();
        if status_code != reqwest::StatusCode::OK {
            if status_code == reqwest::StatusCode::UNPROCESSABLE_ENTITY {
                let querry_response: ValidationError =
                    resp.json::<ValidationError>().map_err(|e| {
                        RoqoqoBackendError::NetworkError {
                            msg: format!("Error parsing ValidationError message {:?}", e),
                        }
                    })?;
                return Err(RoqoqoBackendError::GenericError{msg:
                    format!( "QuantumProgram or metadata could not be parsed by QRyd Web-API Backend. msg: {} type: {}, loc: {:?}",querry_response.detail.msg, querry_response.detail.internal_type, querry_response.detail.loc,  )
            });
            }
            return Err(RoqoqoBackendError::NetworkError {
                msg: format!(
                    "Request to server failed with HTTP status code {:?}",
                    status_code
                ),
            });
        } else {
            Ok(())
        }
    }

    /// Convert the counts returned from the QRyd WebAPI to Qoqo-style registers
    ///
    /// # Arguments
    ///
    /// `counts` - The counts returned from the Qryd WebAPI
    /// `readout` - The name of the readout register. Needs to be specified based on original circuit
    ///             cannont be extrected from returned result
    /// `number_qubits` - The number of measured qubits. Needs to be specified based on original circuit
    ///                   cannont be extrected from returned result
    pub fn counts_to_result(
        counts: ResultCounts,
        readout: String,
        number_qubits: usize,
    ) -> RegisterResult {
        let mut bit_map: HashMap<String, Vec<Vec<bool>>> = HashMap::new();
        let float_map: HashMap<String, Vec<Vec<f64>>> = HashMap::new();
        let complex_map: HashMap<String, Vec<Vec<Complex64>>> = HashMap::new();
        let mut measurement_record: Vec<Vec<bool>> = Vec::new();
        for (measurement, count) in counts.counts.into_iter() {
            let bit_representation: Vec<u8> = hex::decode(
                measurement
                    .strip_prefix("0x")
                    .map(|s| {
                        if s.len() % 2 == 0 {
                            s.to_string()
                        } else {
                            format!("0{}", s)
                        }
                    })
                    .ok_or(RoqoqoBackendError::GenericError {
                        msg: format!(
                            "Cannot parse a measurement result as bit representation {}",
                            measurement.clone()
                        ),
                    })?,
            )
            .map_err(|err| RoqoqoBackendError::GenericError {
                msg: format!(
                    "Cannot parse a measurement result as bit representation {:?}",
                    err
                ),
            })?;
            let qubit_results = bit_representation.view_bits::<Lsb0>();
            let mut tmp_vec: Vec<bool> = (0..number_qubits).into_iter().map(|_| false).collect();
            // only iterating over qubits in number_qubits returns of larger qubits will be ignored
            for (mut_val, tmp_val) in (tmp_vec.iter_mut()).zip(qubit_results.iter()) {
                *mut_val = *tmp_val
            }
            for _ in 0..count {
                measurement_record.push(tmp_vec.clone())
            }
        }
        bit_map.insert(readout, measurement_record);
        Ok((bit_map, float_map, complex_map))
    }
}

impl EvaluatingBackend for APIBackend {
    fn run_circuit_iterator<'a>(
        &self,
        circuit: impl Iterator<Item = &'a Operation>,
    ) -> RegisterResult {
        println!("test");
        let new_circ: Circuit = circuit.cloned().collect();

        let mut readout = "".to_string();
        let mut number_qubits = 0;

        for op in new_circ.iter() {
            if let Operation::DefinitionBit(x) = op {
                let new_readout = x.name().clone();
                if readout == *"" {
                    readout = new_readout;
                    number_qubits = *x.length();
                } else {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: "QRydAPIBAckend does not support more than one readout register"
                            .to_string(),
                    });
                }
            }
        }

        let measurement = ClassicalRegister {
            constant_circuit: None,
            circuits: vec![new_circ],
        };
        let program = QuantumProgram::ClassicalRegister {
            measurement,
            input_parameter_names: vec![],
        };
        let job_loc = self.post_job(program)?;

        let mut test_counter = 0;
        let mut status = "".to_string();
        let mut job_result = QRydJobResult::default();
        let fifteen = time::Duration::from_millis(200);
        dbg!(&status);
        while test_counter < self.timeout && status != "completed" {
            test_counter += 1;
            let job_status = self.get_job_status(job_loc.clone()).unwrap();
            status = job_status.status.clone();
            thread::sleep(fifteen);
            if status == *"completed" {
                job_result = self.get_job_result(job_loc.clone()).unwrap();
            }
        }

        if status == "completed" {
            APIBackend::counts_to_result(job_result.data, readout, number_qubits)
        } else if status == "error" {
            Err(RoqoqoBackendError::GenericError {
                msg: format!("WebAPI returned an error status for the job {}.", job_loc),
            })
        } else if status == "cancelled" {
            Err(RoqoqoBackendError::GenericError {
                msg: format!("Job {} got cancelled.", job_loc),
            })
        } else {
            Err(RoqoqoBackendError::GenericError {
                msg: format!(
                    "WebAPI did not return finished result in timeout: {} * 30s",
                    self.timeout
                ),
            })
        }
    }
}