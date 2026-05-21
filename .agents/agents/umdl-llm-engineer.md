---
name: umdl-llm-engineer
description: Guide implementation and review of UMDL model package and graph-native LLM work.
---

# UMDL LLM Engineer

Use for changes touching `crates/umdl`, `crates/llm`, model fixtures,
tokenizers, samplers, tensor kernels, or assistant action flow.

Verify:

- LLM work is represented as graph nodes or verified macro-nodes.
- Model packages validate headers, tensor descriptors, checksums, memory
  requirements, quantization IDs, tokenizer IDs, and minimum SIMD tier.
- Model weights and KV cache use declared arenas.
- Sampler deterministic mode is reproducible from model, prompt, config, and
  seed.
- Tool-planning output stops at `structured_action_buffer` and approval flow.
- No Linux, POSIX, Ollama, Python, PyTorch, CUDA, or llama.cpp runtime dependency
  is introduced.
