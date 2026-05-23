//! Polling serial operator shell for live milestone verification.
//!
//! This is the first interactive OS surface: after boot reaches `BOOT_OK`, a
//! human or QEMU smoke harness can issue bounded commands over COM1. The shell
//! does not mutate graphs or execute generated code; it exercises existing
//! graph/LLM/storage data paths through explicit operator commands.

use crate::{cpu::SimdTier, serial, ssod};
use graph::{graph_compile_verified, graph_load_from_umod, SOURCE_TRANSFORM_SINK_UMOD};
use llm::{
    assistant::{
        assistant_explain, assistant_retrieve_context, AssistantExplainRequest,
        AssistantRetrievalRequest, GraphExplanationInput, SsodExplanationInput, SsodFaultFamily,
    },
    dispatch::build_dispatch_table,
    quantized::{
        stream_tokens, QuantizedStepConfig, QuantizedStreamBuffers, QuantizedStreamConfig,
        QuantizedStreamState,
    },
    retrieval::{
        retrieve_top_k, RetrievalDocumentRef, RetrievalIndexSnapshot, RetrievalQuery,
        RetrievalResult, RetrievalResultBuffer,
    },
    tokenizer,
    toy_transformer::{generate_text, ToyGenerationConfig, ToyModelMetadata},
};
use umdl::{
    LoadedUmdlModel, SimdTier as ModelSimdTier, TokenizerMetadata, UmdlArenaReservations,
    UmdlHeader, UmdlSectionRange, UmdlSectionRanges, M9_SUPPORTED_ARCHITECTURE_ID,
    UMDL_FORMAT_MAJOR, UMDL_FORMAT_MINOR, UMDL_HEADER_LENGTH,
};

const COMMAND_BYTES: usize = 80;

pub fn run(tier: SimdTier) -> ! {
    if !serial::is_available() {
        ssod::halt_idle();
    }

    serial::write_str("UNBOUNDOS_SHELL_READY\n");
    serial::write_str("unbound> ");

    let mut line = [0u8; COMMAND_BYTES];
    let mut len = 0usize;
    loop {
        if let Some(byte) = serial::read_byte_nonblocking() {
            match byte {
                b'\r' | b'\n' => {
                    serial::write_str("\n");
                    if len != 0 {
                        dispatch(&line[..len], tier);
                    }
                    len = 0;
                    serial::write_str("unbound> ");
                }
                8 | 127 => {
                    len = len.saturating_sub(1);
                }
                byte if byte.is_ascii_control() => {}
                byte => {
                    if len < line.len() {
                        line[len] = byte;
                        len += 1;
                    } else {
                        serial::write_str("\nERR command_too_long\n");
                        len = 0;
                        serial::write_str("unbound> ");
                    }
                }
            }
        } else {
            core::hint::spin_loop();
        }
    }
}

fn dispatch(command: &[u8], tier: SimdTier) {
    match trim_ascii(command) {
        b"help" => serial::write_str(
            "OK help commands=ping,graph,tokenize,toy,quant,retrieve,assistant,ssod,cpu,exit\n",
        ),
        b"ping" => serial::write_str("OK pong\n"),
        b"graph" => command_graph(),
        b"tokenize" => command_tokenize(),
        b"toy" => command_toy(),
        b"quant" => command_quant(),
        b"retrieve" => command_retrieve(),
        b"assistant" => command_assistant(),
        b"ssod" => command_ssod(),
        b"cpu" => {
            serial::write_str("OK cpu tier=");
            serial::write_str(tier.as_str());
            serial::write_str("\n");
        }
        b"exit" => {
            serial::write_str("OK halt\n");
            ssod::halt_idle();
        }
        _ => serial::write_str("ERR unknown_command\n"),
    }
}

fn command_graph() {
    match graph_load_from_umod(SOURCE_TRANSFORM_SINK_UMOD).and_then(|verified| {
        graph_compile_verified(verified).map_err(|_| graph::GraphLoadError::BadSectionTable)
    }) {
        Ok(handle) => {
            let state = handle.display_state();
            serial::write_str("OK graph graph_id=");
            serial::write_hex_u64(state.graph_id());
            serial::write_str(" nodes=");
            serial::write_dec_u64(u64::from(state.node_count()));
            serial::write_str(" wires=");
            serial::write_dec_u64(u64::from(state.wire_count()));
            serial::write_str(" last_completed=");
            write_optional_u32(state.last_completed_node());
            serial::write_str("\n");
        }
        Err(_) => serial::write_str("ERR graph\n"),
    }
}

fn command_tokenize() {
    let mut tokens = [0u32; 16];
    let mut bytes = [0u8; 16];
    match tokenizer::round_trip_raw_bytes(
        TokenizerMetadata::raw_byte_to_token(),
        "hello",
        &mut tokens,
        &mut bytes,
    ) {
        Ok(text) => {
            serial::write_str("OK tokenize text=");
            serial::write_str(text);
            serial::write_str(" tokens=");
            serial::write_dec_u64(5);
            serial::write_str("\n");
        }
        Err(_) => serial::write_str("ERR tokenize\n"),
    }
}

fn command_toy() {
    let mut prompt_tokens = [0u32; 16];
    let mut generated_tokens = [0u32; 8];
    let mut output = [0u8; 16];
    match generate_text(
        ToyModelMetadata::m8_toy(),
        ToyGenerationConfig::deterministic(6, 7),
        TokenizerMetadata::raw_byte_to_token(),
        "OS",
        &mut prompt_tokens,
        &mut generated_tokens,
        &mut output,
    ) {
        Ok(text) => {
            serial::write_str("OK toy text=");
            serial::write_str(text);
            serial::write_str("\n");
        }
        Err(_) => serial::write_str("ERR toy\n"),
    }
}

fn command_quant() {
    let kernels = build_dispatch_table(ModelSimdTier::Scalar);
    let prompt = [u32::from(b'O'), u32::from(b'S')];
    let input = [2, -3, 4];
    let weights = [1, 2, 3, -4, 5, -6, 3, -2, 1];
    let bias = [7, -8, 3];
    let mut logits = [0i32; 3];
    let mut output = [0u32; 3];
    let mut state = QuantizedStreamState::new();
    let mut buffers = QuantizedStreamBuffers {
        prompt_tokens: &prompt,
        projection_input: &input,
        projection_weights: &weights,
        projection_bias: Some(&bias),
        logits: &mut logits,
        output_tokens: &mut output,
    };

    match stream_tokens(
        model_view(),
        &kernels,
        QuantizedStreamConfig {
            max_new_tokens: 3,
            step: QuantizedStepConfig {
                candidate_token_base: 65,
                candidate_count: 3,
            },
        },
        &mut state,
        &mut buffers,
    ) {
        Ok(count) => {
            serial::write_str("OK quant tokens=");
            for (index, token) in output[..count].iter().enumerate() {
                if index != 0 {
                    serial::write_byte(b',');
                }
                serial::write_dec_u64(u64::from(*token));
            }
            serial::write_str(" last=");
            serial::write_dec_u64(u64::from(state.last_token));
            serial::write_str("\n");
        }
        Err(_) => serial::write_str("ERR quant\n"),
    }
}

fn command_retrieve() {
    let Ok(first) =
        RetrievalDocumentRef::new("index:spec-13.1", "Spec", "assistant searches local docs")
    else {
        serial::write_str("ERR retrieve_doc\n");
        return;
    };
    let Ok(second) = RetrievalDocumentRef::new("blob:boot", "Boot", "serial heartbeat graph")
    else {
        serial::write_str("ERR retrieve_doc\n");
        return;
    };
    let docs = [first, second];
    let Ok(index) = RetrievalIndexSnapshot::new(&docs) else {
        serial::write_str("ERR retrieve_index\n");
        return;
    };
    let Ok(query) = RetrievalQuery::new("assistant docs") else {
        serial::write_str("ERR retrieve_query\n");
        return;
    };
    let mut result_storage = [RetrievalResult::new(0, 0, "index:empty").unwrap(); 2];
    let mut results = RetrievalResultBuffer::new(&mut result_storage);
    match retrieve_top_k(&index, &query, 2, &mut results) {
        Ok(count) => {
            let mut context = [0u8; 256];
            let Ok(response) = assistant_retrieve_context(
                AssistantRetrievalRequest {
                    index: &index,
                    results: results.results(),
                    proposed_action: None,
                },
                &mut context,
                None,
            ) else {
                serial::write_str("ERR retrieve_context\n");
                return;
            };
            serial::write_str("OK retrieve count=");
            serial::write_dec_u64(count as u64);
            serial::write_str(" context_len=");
            serial::write_dec_u64(u64::from(response.context_len));
            if let Some(result) = results.results().first() {
                serial::write_str(" top=");
                write_ascii(result.resource_ref_bytes());
            }
            serial::write_str("\n");
        }
        Err(_) => serial::write_str("ERR retrieve\n"),
    }
}

fn command_assistant() {
    let input = GraphExplanationInput::new(0x0053_5453, 3, 2, None, Some(3));
    let mut output = [0u8; 128];
    match assistant_explain(
        AssistantExplainRequest::Graph {
            input: &input,
            proposed_action: None,
        },
        &mut output,
        None,
    ) {
        Ok(response) => {
            let text_len = response.explanation_len as usize;
            serial::write_str("OK assistant ");
            write_ascii(&output[..text_len.min(output.len())]);
            serial::write_str("\n");
        }
        Err(_) => serial::write_str("ERR assistant\n"),
    }
}

fn command_ssod() {
    let Ok(input) = SsodExplanationInput::new(
        SsodFaultFamily::CpuException,
        14,
        "page_fault",
        0xffff_8000_0000_1234,
        Some(2),
    ) else {
        serial::write_str("ERR ssod_input\n");
        return;
    };
    let mut output = [0u8; 128];
    match assistant_explain(
        AssistantExplainRequest::Ssod {
            input: &input,
            proposed_action: None,
        },
        &mut output,
        None,
    ) {
        Ok(response) => {
            serial::write_str("OK ssod ");
            write_ascii(&output[..(response.explanation_len as usize).min(output.len())]);
            serial::write_str("\n");
        }
        Err(_) => serial::write_str("ERR ssod\n"),
    }
}

fn model_view() -> LoadedUmdlModel {
    let header = UmdlHeader {
        magic: *b"UMDL",
        format_major: UMDL_FORMAT_MAJOR,
        format_minor: UMDL_FORMAT_MINOR,
        header_length: UMDL_HEADER_LENGTH,
        architecture_id: M9_SUPPORTED_ARCHITECTURE_ID,
        quantization_scheme_id: 0,
        tensor_count: 1,
        tokenizer_section_offset: 160,
        tokenizer_section_length: 72,
        tensor_section_offset: 240,
        tensor_section_length: 48,
        weight_blob_offset: 320,
        weight_blob_length: 16,
        checksum_section_offset: 400,
        checksum_section_length: 24,
        required_memory_bytes: 16,
        required_scratch_bytes: 8,
        required_kv_cache_bytes_per_token: 2,
        max_context_tokens: 32,
        vocabulary_size: 256,
        layer_count: 1,
        hidden_size: 8,
        attention_head_count: 1,
        minimum_simd_tier: ModelSimdTier::Scalar as u32,
        model_stable_id: 0x0000_0000_000a_0001,
        header_checksum: 0,
    };
    LoadedUmdlModel {
        header,
        tokenizer: TokenizerMetadata::raw_byte_to_token(),
        tensor_count: 1,
        ranges: UmdlSectionRanges {
            tokenizer: UmdlSectionRange {
                offset: 160,
                length: 72,
            },
            tensor: UmdlSectionRange {
                offset: 240,
                length: 48,
            },
            weight_blob: UmdlSectionRange {
                offset: 320,
                length: 16,
            },
            checksum: UmdlSectionRange {
                offset: 400,
                length: 24,
            },
        },
        reservations: UmdlArenaReservations {
            model_weight_bytes: 16,
            scratch_bytes: 8,
            kv_cache_bytes_per_token: 2,
            max_context_tokens: 32,
        },
        active_simd_tier: ModelSimdTier::Scalar,
    }
}

fn trim_ascii(mut bytes: &[u8]) -> &[u8] {
    while let Some((first, rest)) = bytes.split_first() {
        if first.is_ascii_whitespace() {
            bytes = rest;
        } else {
            break;
        }
    }
    while let Some((last, rest)) = bytes.split_last() {
        if last.is_ascii_whitespace() {
            bytes = rest;
        } else {
            break;
        }
    }
    bytes
}

fn write_optional_u32(value: Option<u32>) {
    if let Some(value) = value {
        serial::write_dec_u64(u64::from(value));
    } else {
        serial::write_str("none");
    }
}

fn write_ascii(bytes: &[u8]) {
    for byte in bytes {
        if byte.is_ascii() && !byte.is_ascii_control() {
            serial::write_byte(*byte);
        } else if *byte == b'\n' {
            serial::write_byte(b' ');
        }
    }
}
